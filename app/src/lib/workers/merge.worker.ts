/**
 * Web Worker for image merging.
 *
 * Loads the WASM engine and processes merge requests off the main thread.
 */

import type { MergeRequest, MergeSuccess, MergeError } from './types';

const normalizeBase = (base: string): string => (base.endsWith('/') ? base : `${base}/`);
const deriveBaseFromWorker = (): string => {
	const path = self.location.pathname;
	const appRootIndex = path.indexOf('/_app/');
	if (appRootIndex > 0) {
		return path.slice(0, appRootIndex);
	}
	return '';
};
const resolveBase = (basePath?: string): string => {
	if (basePath && basePath.length > 0) {
		return normalizeBase(basePath);
	}
	const derived = deriveBaseFromWorker();
	return normalizeBase(derived || '/');
};

let runtimeBase = resolveBase();
const resolveStaticUrl = (path: string): string =>
	new URL(`${runtimeBase}${path}`, self.location.origin).toString();

// Dynamic import path - resolved at runtime relative to app base
const getWasmModuleUrl = (): string => resolveStaticUrl('wasm/merge_images_engine.js');

// WASM module interface
interface WasmModule {
	default: (path: string) => Promise<void>;
	greet: () => string;
	merge_images: (images: Uint8Array[], options: Record<string, unknown>) => Uint8Array;
}

const isWasmModule = (value: unknown): value is WasmModule => {
	if (!value || typeof value !== 'object') {
		return false;
	}

	const module = value as Record<string, unknown>;

	return (
		typeof module.default === 'function' &&
		typeof module.greet === 'function' &&
		typeof module.merge_images === 'function'
	);
};

let wasmModule: WasmModule | null = null;
let initPromise: Promise<void> | null = null;

/**
 * Initialize the WASM module (lazy, once).
 */
async function ensureInitialized(basePath?: string): Promise<void> {
	if (wasmModule) return;

	initPromise ??= (async () => {
		runtimeBase = resolveBase(basePath);
		// Import the WASM module
		const module = (await import(/* @vite-ignore */ getWasmModuleUrl())) as unknown;

		if (!isWasmModule(module)) {
			throw new Error('Invalid WASM module shape');
		}

		// Initialize WASM with the .wasm file path
		await module.default(resolveStaticUrl('wasm/merge_images_engine_bg.wasm'));
		wasmModule = module;
	})();

	await initPromise;
}

/**
 * Parse PNG header to extract width and height without decoding the full image.
 * PNG format: https://www.w3.org/TR/PNG/
 *
 * @param bytes - PNG image bytes
 * @returns { width, height } or null if not a valid PNG
 */
function parsePngDimensions(bytes: Uint8Array): { width: number; height: number } | null {
	// PNG signature: 89 50 4E 47 0D 0A 1A 0A
	const PNG_SIGNATURE = [0x89, 0x50, 0x4e, 0x47, 0x0d, 0x0a, 0x1a, 0x0a];
	if (bytes.length < 24) return null;

	// Check PNG signature
	for (let i = 0; i < PNG_SIGNATURE.length; i++) {
		if (bytes[i] !== PNG_SIGNATURE[i]) return null;
	}

	// IHDR chunk type: 49 48 44 52
	const IHDR = [0x49, 0x48, 0x44, 0x52];
	for (let i = 12; i < 12 + 4; i++) {
		if (bytes[i] !== IHDR[i - 12]) return null;
	}

	const view = new DataView(bytes.buffer, bytes.byteOffset, bytes.byteLength);

	// Width (4 bytes, big-endian) at offset 16
	const width = view.getUint32(16, false);

	// Height (4 bytes, big-endian) at offset 20
	const height = view.getUint32(20, false);

	return { width, height };
}

/**
 * Handle incoming merge requests.
 */
self.onmessage = async (event: MessageEvent<MergeRequest>) => {
	const request = event.data;

	try {
		// Initialize WASM if needed
		await ensureInitialized(request.basePath);

		if (!wasmModule) {
			throw new Error('WASM module failed to initialize');
		}

		// Read all files as ArrayBuffers
		const buffers = await Promise.all(request.files.map((file) => file.arrayBuffer()));

		// Convert to Uint8Array[]
		const arrays = buffers.map((buf) => new Uint8Array(buf));

		// Build options object for WASM
		const wasmOptions: Record<string, unknown> = {
			direction: request.options.direction,
			background: request.options.background
		};

		if (request.maxOutPixels !== undefined) {
			wasmOptions.maxOutPixels = request.maxOutPixels;
		}

		// Call WASM merge function
		const result = wasmModule.merge_images(arrays, wasmOptions);

		// Get the underlying ArrayBuffer for transfer
		const resultBuffer = new Uint8Array(result).buffer;

		// Parse PNG header for dimensions (fast, no decoding)
		const dimensions = parsePngDimensions(new Uint8Array(resultBuffer));
		if (!dimensions) {
			throw new Error('Failed to parse output PNG dimensions');
		}

		// Send success response with transferable buffer
		const response: MergeSuccess = {
			type: 'MERGE_SUCCESS',
			bytes: resultBuffer,
			width: dimensions.width,
			height: dimensions.height,
			mime: 'image/png'
		};

		postMessage(response, { transfer: [resultBuffer] });
	} catch (error: unknown) {
		// Handle WASM errors (structured error objects from Rust)
		const details: NonNullable<MergeError['details']> = {};
		const response: MergeError = {
			type: 'MERGE_ERROR',
			code: 'INTERNAL_ERROR',
			message: 'Unknown error',
			details
		};

		if (error && typeof error === 'object') {
			const err = error as Record<string, unknown>;

			// WASM errors have code, message, and optional details
			if ('code' in err && typeof err.code === 'string') {
				response.code = err.code;
			}
			if ('message' in err && typeof err.message === 'string') {
				response.message = err.message;
			}

			// Copy error details
			if ('fileIndex' in err && typeof err.fileIndex === 'number') {
				details.fileIndex = err.fileIndex;
			}
			if ('width' in err && typeof err.width === 'number') {
				details.width = err.width;
			}
			if ('height' in err && typeof err.height === 'number') {
				details.height = err.height;
			}
			if ('outPixels' in err && typeof err.outPixels === 'number') {
				details.outPixels = err.outPixels;
			}
			if ('maxOutPixels' in err && typeof err.maxOutPixels === 'number') {
				details.maxOutPixels = err.maxOutPixels;
			}
		} else if (error instanceof Error) {
			response.message = error.message;
		}

		self.postMessage(response);
	}
};

// Signal that the worker is ready
self.postMessage({ type: 'WORKER_READY' });
