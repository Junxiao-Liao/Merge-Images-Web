/**
 * Supported image formats for the merge engine.
 * These are the MIME types that the WASM engine can decode.
 */
export const SUPPORTED_IMAGE_TYPES = [
	'image/png',
	'image/jpeg',
	'image/gif',
	'image/webp',
	'image/tiff'
] as const;

/**
 * HEIC/HEIF types that need early rejection with user-friendly error.
 * These formats are common on iOS but not supported by the engine.
 */
export const HEIC_TYPES = ['image/heic', 'image/heif'] as const;

/**
 * Accept string for file input elements.
 */
export const ACCEPT_STRING = SUPPORTED_IMAGE_TYPES.join(',');

/**
 * Check if a file is a supported image type.
 */
export function isSupportedImage(file: File): boolean {
	return (SUPPORTED_IMAGE_TYPES as readonly string[]).includes(file.type);
}

/**
 * Check if a file is a HEIC/HEIF file.
 * Also checks file extension as fallback since some browsers don't report HEIC MIME type.
 */
export function isHeicFile(file: File): boolean {
	if ((HEIC_TYPES as readonly string[]).includes(file.type)) {
		return true;
	}
	// Fallback: check extension for when MIME type is not reported
	const ext = file.name.toLowerCase().split('.').pop();
	return ext === 'heic' || ext === 'heif';
}

/**
 * Filter files and categorize them as supported, HEIC (rejected), or other unsupported.
 */
export function categorizeFiles(files: File[]): {
	supported: File[];
	heicFiles: File[];
	unsupported: File[];
} {
	const supported: File[] = [];
	const heicFiles: File[] = [];
	const unsupported: File[] = [];

	for (const file of files) {
		if (isSupportedImage(file)) {
			supported.push(file);
		} else if (isHeicFile(file)) {
			heicFiles.push(file);
		} else {
			unsupported.push(file);
		}
	}

	return { supported, heicFiles, unsupported };
}
