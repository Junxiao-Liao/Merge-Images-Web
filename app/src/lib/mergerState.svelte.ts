import {
	type ImageFile,
	type MergeState,
	type Direction,
	type BackgroundColor,
	type MergeError
} from '$lib/types';
import { DEFAULT_OPTIONS } from '$lib/workers/types';
import { createImageFile, revokeImageFile, revokeAllImageFiles } from '$lib/utils/thumbnails';
import { categorizeFiles } from '$lib/utils/formats';

class MergerStateStore {
	// State
	images = $state<ImageFile[]>([]);
	direction = $state<Direction>(DEFAULT_OPTIONS.direction);
	background = $state<BackgroundColor>(DEFAULT_OPTIONS.background);
	overlapSensitivity = $state<number>(DEFAULT_OPTIONS.overlapSensitivity);
	mergeState = $state<MergeState>({ status: 'idle' });

	// Derived
	canMerge = $derived(this.images.length >= 2 && this.mergeState.status !== 'processing');
	isProcessing = $derived(this.mergeState.status === 'processing');
	hasResult = $derived(this.mergeState.status === 'success');
	hasError = $derived(this.mergeState.status === 'error');

	// Actions
	addFiles(files: File[]) {
		const { supported, heicFiles } = categorizeFiles(files);

		let error: string | null = null;
		if (heicFiles.length > 0) {
			const names = heicFiles.map((f) => f.name).join(', ');
			error = `HEIC/HEIF format is not supported: ${names}. Please convert to PNG or JPEG first.`;
		}

		if (supported.length > 0) {
			const newImages = supported.map(createImageFile);
			this.images = [...this.images, ...newImages];
		}

		return { error };
	}

	removeImage(id: string) {
		const image = this.images.find((img) => img.id === id);
		if (image) {
			revokeImageFile(image);
		}
		this.images = this.images.filter((img) => img.id !== id);
	}

	reorderImages(newImages: ImageFile[]) {
		this.images = newImages;
	}

	setDirection(d: Direction) {
		this.direction = d;
	}

	setBackground(bg: BackgroundColor) {
		this.background = bg;
	}

	setOverlapSensitivity(value: number) {
		this.overlapSensitivity = Math.max(0, Math.min(100, value));
	}

	setMergeState(state: MergeState) {
		// If we have a previous success state and we are setting a new one, revoke the old URL
		// ONLY if we are actually replacing it (e.g. starting new merge or clearing)
		// Note: We don't revoke if just navigating away, so we do it here carefully.
		if (this.mergeState.status === 'success' && state.status !== 'success') {
			// If we are resetting to idle/processing, we can revoke existing result
			URL.revokeObjectURL(this.mergeState.url);
		}
		this.mergeState = state;
	}

	setError(message: string) {
		const error: MergeError = {
			type: 'MERGE_ERROR',
			code: 'UNSUPPORTED_FORMAT',
			message
		};
		this.mergeState = { status: 'error', error };
	}

	reset() {
		revokeAllImageFiles(this.images);
		if (this.mergeState.status === 'success') {
			URL.revokeObjectURL(this.mergeState.url);
		}
		this.images = [];
		this.mergeState = { status: 'idle' };
	}
}

export const mergerState = new MergerStateStore();
