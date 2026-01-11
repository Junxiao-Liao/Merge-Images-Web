/**
 * Device classification for adaptive pixel limits.
 */

export type DeviceClass = 'mobile' | 'desktop';

/**
 * Detect device class based on viewport and touch capability.
 */
export function detectDeviceClass(): DeviceClass {
	if (typeof window === 'undefined') return 'desktop';

	const viewport = Math.min(window.innerWidth, window.innerHeight);
	const hasTouchPoints = navigator.maxTouchPoints > 0;
	const isMobileViewport = viewport < 900;

	// Mobile if: small viewport AND touch capability
	if (isMobileViewport && hasTouchPoints) {
		return 'mobile';
	}

	return 'desktop';
}
