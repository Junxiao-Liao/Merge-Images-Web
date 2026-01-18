/**
 * Download utilities with Safari/iOS fallback using Web Share API.
 */

/**
 * Result of download attempt.
 */
export interface DownloadResult {
	success: boolean;
	usedFallback: boolean;
	method: 'download' | 'share' | 'new-tab';
}

/**
 * Detect if running on iOS where download attribute doesn't work.
 */
function isIOS(): boolean {
	if (typeof navigator === 'undefined') return false;
	const ua = navigator.userAgent;
	return /iPad|iPhone|iPod/.test(ua);
}

/**
 * Check if Web Share API supports sharing the given file.
 */
function canShareFile(file: File): boolean {
	if (typeof navigator === 'undefined') return false;
	// canShare may not exist on all browsers (use 'in' to avoid TS thinking it always exists)
	if (!('canShare' in navigator)) return false;
	try {
		return navigator.canShare({ files: [file] });
	} catch {
		return false;
	}
}

/**
 * Share a blob via Web Share API.
 * Returns true if share was initiated (including user cancel).
 */
async function shareBlob(blob: Blob, filename: string): Promise<boolean> {
	const file = new File([blob], filename, { type: blob.type });

	if (!canShareFile(file)) {
		return false;
	}

	try {
		await navigator.share({ files: [file], title: 'Merged Image' });
		return true;
	} catch (error) {
		// User cancelled - still considered success (dialog was shown)
		if (error instanceof Error && error.name === 'AbortError') {
			return true;
		}
		return false;
	}
}

/**
 * Download a blob as a file.
 * On iOS, attempts Web Share API first, then falls back to opening in new tab.
 * On other browsers, uses standard anchor download.
 */
export async function downloadBlob(blob: Blob, filename: string): Promise<DownloadResult> {
	if (isIOS()) {
		// Try Web Share API first
		const shared = await shareBlob(blob, filename);
		if (shared) {
			return { success: true, usedFallback: false, method: 'share' };
		}

		// Fallback: open in new tab
		const url = URL.createObjectURL(blob);
		window.open(url, '_blank');
		return { success: true, usedFallback: true, method: 'new-tab' };
	}

	// Standard download via anchor element
	const url = URL.createObjectURL(blob);
	const link = document.createElement('a');
	link.href = url;
	link.download = filename;
	link.style.display = 'none';
	document.body.appendChild(link);
	link.click();

	// Cleanup after a short delay
	setTimeout(() => {
		document.body.removeChild(link);
		URL.revokeObjectURL(url);
	}, 100);

	return { success: true, usedFallback: false, method: 'download' };
}
