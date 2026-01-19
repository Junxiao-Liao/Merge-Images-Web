/**
 * Frontend type definitions.
 */

import type { BackgroundColor, Direction, MergeError, MergeOptions } from './workers/types';

/** Represents an image loaded into the UI. */
export interface ImageFile {
	id: string;
	file: File;
	name: string;
	thumbnailUrl: string;
}

/** Merge processing state. */
export type MergeState =
	| { status: 'idle' }
	| { status: 'processing'; stage: string; percent: number }
	| { status: 'success'; blob: Blob; url: string; width: number; height: number }
	| { status: 'error'; error: MergeError };

/** Re-export worker types for convenience. */
export type { BackgroundColor, Direction, MergeError, MergeOptions };
