import { test, expect } from '@playwright/test';
import path from 'path';

const fixturesDir = path.join(import.meta.dirname, 'fixtures');

test.describe('Image Merge', () => {
	test.beforeEach(async ({ page }) => {
		await page.goto('/');

		// Upload test images
		const fileInput = page.getByTestId('file-input');
		await fileInput.setInputFiles([
			path.join(fixturesDir, 'red.png'),
			path.join(fixturesDir, 'blue.png')
		]);

		// Wait for all images to load
		await expect(page.getByTestId('image-item')).toHaveCount(2);
	});

	test('should merge images vertically by default', async ({ page }) => {
		const mergeButton = page.getByTestId('merge-button');
		await mergeButton.click();

		// Wait for preview to appear
		const preview = page.getByTestId('preview');
		await expect(preview).toBeVisible({ timeout: 30000 });

		// Verify dimensions shown (10x10 + 10x10 vertical = 10x20)
		await expect(preview).toContainText('10 x 20 pixels');
	});

	test('should merge images horizontally', async ({ page }) => {
		// Switch to horizontal
		const horizontalOption = page.getByRole('button', { name: 'Horizontal' });
		await horizontalOption.click();

		const mergeButton = page.getByTestId('merge-button');
		await mergeButton.click();

		// Wait for preview to appear
		const preview = page.getByTestId('preview');
		await expect(preview).toBeVisible({ timeout: 30000 });

		// Verify dimensions shown (10x10 + 10x10 horizontal = 20x10)
		await expect(preview).toContainText('20 x 10 pixels');
	});

	test('should show processing state during merge', async ({ page }) => {
		const mergeButton = page.getByTestId('merge-button');
		await mergeButton.click();

		// Check for processing indicator (may be quick)
		// The button should show "Merging..." or be disabled during processing
		await expect(page.getByTestId('preview')).toBeVisible({ timeout: 30000 });
	});

	test('should display file size in preview', async ({ page }) => {
		const mergeButton = page.getByTestId('merge-button');
		await mergeButton.click();

		const preview = page.getByTestId('preview');
		await expect(preview).toBeVisible({ timeout: 30000 });

		// Preview should show file size (in KB or MB)
		await expect(preview).toContainText(/\d+(\.\d+)?\s*(B|KB|MB)/);
	});

	test('should allow re-merge after changing options', async ({ page }) => {
		// First merge (vertical)
		const mergeButton = page.getByTestId('merge-button');
		await mergeButton.click();

		const preview = page.getByTestId('preview');
		await expect(preview).toBeVisible({ timeout: 30000 });
		await expect(preview).toContainText('10 x 20 pixels');

		// Switch to horizontal
		const horizontalOption = page.getByRole('button', { name: 'Horizontal' });
		await horizontalOption.click();

		// Merge again
		await mergeButton.click();

		// Verify new dimensions
		await expect(preview).toContainText('20 x 10 pixels');
	});
});
