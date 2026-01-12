/**
 * Worker manager for WASM merge engine communication.
 */

import { resolve } from '$app/paths';
import type { MergeOptions, MergeResponse, MergeSuccess, MergeError } from '$lib/workers/types';

export type SuccessCallback = (result: MergeSuccess) => void;
export type ErrorCallback = (error: MergeError) => void;

let worker: Worker | null = null;
let workerReady = false;
let pendingCallbacks: { onSuccess: SuccessCallback; onError: ErrorCallback } | null = null;

/**
 * Get or create the worker instance.
 */
function getWorker(): Worker {
	if (!worker) {
		worker = new Worker(new URL('$lib/workers/merge.worker.ts', import.meta.url), {
			type: 'module'
		});

		worker.onmessage = (event: MessageEvent<MergeResponse | { type: 'WORKER_READY' }>) => {
			const response = event.data;

			if (response.type === 'WORKER_READY') {
				workerReady = true;
				return;
			}

			if (!pendingCallbacks) return;

			switch (response.type) {
				case 'MERGE_SUCCESS':
					pendingCallbacks.onSuccess(response);
					pendingCallbacks = null;
					break;
				case 'MERGE_ERROR':
					pendingCallbacks.onError(response);
					pendingCallbacks = null;
					break;
			}
		};

		worker.onerror = (error) => {
			console.error('Worker error:', error);
			if (pendingCallbacks) {
				pendingCallbacks.onError({
					type: 'MERGE_ERROR',
					code: 'INTERNAL_ERROR',
					message: 'Worker crashed unexpectedly'
				});
				pendingCallbacks = null;
			}
		};
	}

	return worker;
}

/**
 * Wait for worker to be ready.
 */
async function waitForReady(): Promise<void> {
	getWorker();
	if (workerReady) return;

	return new Promise((resolve) => {
		const checkReady = () => {
			if (workerReady) {
				resolve();
			} else {
				setTimeout(checkReady, 10);
			}
		};
		checkReady();
	});
}

export interface MergeCallbacks {
	onSuccess: SuccessCallback;
	onError: ErrorCallback;
}

function normalizeOptions(options: MergeOptions): MergeOptions {
	return {
		direction: options.direction,
		background: {
			r: options.background.r,
			g: options.background.g,
			b: options.background.b,
			a: options.background.a
		}
	};
}

/**
 * Start a merge operation.
 */
export async function mergeImages(
	files: File[],
	options: MergeOptions,
	callbacks: MergeCallbacks
): Promise<void> {
	await waitForReady();

	const w = getWorker();

	// Store callbacks for response handling
	pendingCallbacks = callbacks;

	// Send merge request
	w.postMessage({
		type: 'MERGE_REQUEST',
		files: [...files],
		options: normalizeOptions(options),
		basePath: resolve('/')
	});
}

/**
 * Check if a merge is currently in progress.
 */
export function isMerging(): boolean {
	return pendingCallbacks !== null;
}
