/**
 * Adaptive output pixel limits based on device capability.
 */

import type { DeviceClass } from './deviceClass';

interface NavigatorWithMemory extends Navigator {
	deviceMemory?: number;
}

/**
 * Compute maximum output pixels based on device class and available memory.
 * Returns undefined for desktop (no limit enforced).
 */
export function computeMaxOutPixels(deviceClass: DeviceClass): number | undefined {
	if (deviceClass === 'desktop') {
		// No limit on desktop per FR-5.3, but add a safety ceiling
		return 100_000_000; // 100M pixels safety limit
	}

	// Mobile limits based on deviceMemory
	const nav = navigator as NavigatorWithMemory;
	const memory = nav.deviceMemory;

	if (memory === undefined) {
		// Safari fallback - assume moderate capability
		return 12_000_000;
	}

	if (memory <= 2) return 8_000_000;
	if (memory <= 4) return 12_000_000;
	return 16_000_000;
}
