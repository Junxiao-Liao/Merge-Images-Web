/**
 * Worker message protocol types for the merge engine.
 */

/** Background color with RGBA components (0-255). */
export interface BackgroundColor {
	r: number;
	g: number;
	b: number;
	a: number;
}

/** Merge direction. */
export type Direction = 'vertical' | 'horizontal';

/** Merge options. */
export interface MergeOptions {
	direction: Direction;
	background: BackgroundColor;
}

/** Request message sent to the worker. */
export interface MergeRequest {
	type: 'MERGE_REQUEST';
	files: File[];
	options: MergeOptions;
	maxOutPixels?: number;
}

/** Success response from the worker. */
export interface MergeSuccess {
	type: 'MERGE_SUCCESS';
	bytes: ArrayBuffer;
	width: number;
	height: number;
	mime: 'image/png';
}

/** Error response from the worker. */
export interface MergeError {
	type: 'MERGE_ERROR';
	code: string;
	message: string;
	details?: {
		fileIndex?: number;
		fileName?: string;
		width?: number;
		height?: number;
		outPixels?: number;
		maxOutPixels?: number;
	};
}

/** Progress update from the worker (optional). */
export interface MergeProgress {
	type: 'MERGE_PROGRESS';
	stage: string;
	percent: number;
}

/** All possible worker response types. */
export type MergeResponse = MergeSuccess | MergeError | MergeProgress;

/** All possible worker message types. */
export type WorkerMessage = MergeRequest | MergeResponse;

/** Default merge options. */
export const DEFAULT_OPTIONS: MergeOptions = {
	direction: 'vertical',
	background: { r: 255, g: 255, b: 255, a: 255 }
};
