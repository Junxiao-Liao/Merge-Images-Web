import { test, expect } from '@playwright/test';
import path from 'path';

const fixturesDir = path.join(import.meta.dirname, 'fixtures');

test.describe('Download', () => {
	test.beforeEach(async ({ page }) => {
		await page.goto('/');

		// Upload test images
		const fileInput = page.getByTestId('file-input');
		await fileInput.setInputFiles([
			path.join(fixturesDir, 'red.png'),
			path.join(fixturesDir, 'blue.png')
		]);

		// Wait for images to load and merge
		await expect(page.getByTestId('image-item')).toHaveCount(2);

		const mergeButton = page.getByTestId('merge-button');
		await mergeButton.click();

		// Wait for preview to appear
		await expect(page.getByTestId('preview')).toBeVisible({ timeout: 30000 });
	});

	test('should show download button after merge', async ({ page }) => {
		const downloadButton = page.getByTestId('download-button');
		await expect(downloadButton).toBeVisible();
		await expect(downloadButton).toContainText('Download merged.png');
	});

	test('should trigger download on button click', async ({ page }) => {
		const downloadButton = page.getByTestId('download-button');

		// Set up download handler
		const downloadPromise = page.waitForEvent('download');

		await downloadButton.click();

		// Verify download was triggered
		const download = await downloadPromise;
		expect(download.suggestedFilename()).toBe('merged.png');
	});

	test('should show preview image', async ({ page }) => {
		const preview = page.getByTestId('preview');
		const image = preview.locator('img');

		await expect(image).toBeVisible();
		await expect(image).toHaveAttribute('alt', 'Merged result');
	});
});
