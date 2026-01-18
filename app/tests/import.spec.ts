import { test, expect } from '@playwright/test';
import path from 'path';

test.describe('Image Import', () => {
	test.beforeEach(async ({ page }) => {
		await page.goto('/');
	});

	test('should display empty state initially', async ({ page }) => {
		const dropZone = page.getByTestId('drop-zone');
		await expect(dropZone).toBeVisible();
		await expect(dropZone).toContainText('Drop images here or click to select');
		await expect(dropZone).toContainText('PNG, JPG, GIF, WebP, TIFF supported');
	});

	test('should import images via file picker', async ({ page }) => {
		const fileInput = page.getByTestId('file-input');

		// Upload test images
		await fileInput.setInputFiles([
			path.join(__dirname, 'fixtures/red.png'),
			path.join(__dirname, 'fixtures/blue.png')
		]);

		// Verify images appear in the list
		const imageItems = page.getByTestId('image-item');
		await expect(imageItems).toHaveCount(2);

		// Verify drop zone is replaced with image list
		await expect(page.getByTestId('drop-zone')).not.toBeVisible();
	});

	test('should show merge button disabled with less than 2 images', async ({ page }) => {
		const mergeButton = page.getByTestId('merge-button');
		await expect(mergeButton).toBeDisabled();
		await expect(mergeButton).toContainText('Add at least 2 images');
	});

	test('should enable merge button with 2+ images', async ({ page }) => {
		const fileInput = page.getByTestId('file-input');
		await fileInput.setInputFiles([
			path.join(__dirname, 'fixtures/red.png'),
			path.join(__dirname, 'fixtures/blue.png')
		]);

		const mergeButton = page.getByTestId('merge-button');
		await expect(mergeButton).toBeEnabled();
		await expect(mergeButton).toContainText('Merge 2 Images');
	});

	test('should allow adding more images after initial import', async ({ page }) => {
		const fileInput = page.getByTestId('file-input');
		await fileInput.setInputFiles([path.join(__dirname, 'fixtures/red.png')]);

		// Wait for image to appear
		await expect(page.getByTestId('image-item')).toHaveCount(1);

		// Add more via the "Add More" button
		const addMoreButton = page.getByRole('button', { name: 'Add More Images' });
		await expect(addMoreButton).toBeVisible();

		// Find the hidden input for add more
		const addMoreInput = page.locator('input[type="file"]').nth(0);
		await addMoreInput.setInputFiles([path.join(__dirname, 'fixtures/blue.png')]);

		// Verify total count
		await expect(page.getByTestId('image-item')).toHaveCount(2);
	});
});
