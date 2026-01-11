/**
 * Download utilities with Safari/iOS fallback.
 */

/**
 * Result of download attempt.
 */
export interface DownloadResult {
	success: boolean;
	usedFallback: boolean;
}

/**
 * Detect if running on iOS Safari where download attribute doesn't work.
 */
function isIOSSafari(): boolean {
	if (typeof navigator === 'undefined') return false;
	const ua = navigator.userAgent;
	const isIOS = /iPad|iPhone|iPod/.test(ua);
	const isSafari = /^((?!chrome|android).)*safari/i.test(ua);
	return isIOS && isSafari;
}

/**
 * Download a blob as a file. Falls back to opening in new tab on iOS Safari.
 */
export function downloadBlob(blob: Blob, filename: string): DownloadResult {
	const url = URL.createObjectURL(blob);

	if (isIOSSafari()) {
		// Fallback: open in new tab with instructions
		window.open(url, '_blank');
		return { success: true, usedFallback: true };
	}

	// Standard download via anchor element
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

	return { success: true, usedFallback: false };
}
