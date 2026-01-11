/**
 * Thumbnail generation utilities.
 */

import type { ImageFile } from '$lib/types';

/**
 * Generate a unique ID for an image file.
 */
function generateId(): string {
	return crypto.randomUUID();
}

/**
 * Create an ImageFile from a File object.
 * Creates an object URL for thumbnail display.
 */
export function createImageFile(file: File): ImageFile {
	return {
		id: generateId(),
		file,
		name: file.name,
		thumbnailUrl: URL.createObjectURL(file)
	};
}

/**
 * Cleanup object URLs to prevent memory leaks.
 */
export function revokeImageFile(image: ImageFile): void {
	URL.revokeObjectURL(image.thumbnailUrl);
}

/**
 * Cleanup multiple image files.
 */
export function revokeAllImageFiles(images: ImageFile[]): void {
	images.forEach(revokeImageFile);
}
