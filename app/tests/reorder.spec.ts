import { test, expect } from '@playwright/test';
import path from 'path';

const fixturesDir = path.join(import.meta.dirname, 'fixtures');

test.describe('Image Reorder', () => {
	test.beforeEach(async ({ page }) => {
		await page.goto('/');

		// Upload test images
		const fileInput = page.getByTestId('file-input');
		await fileInput.setInputFiles([
			path.join(fixturesDir, 'red.png'),
			path.join(fixturesDir, 'blue.png'),
			path.join(fixturesDir, 'green.png')
		]);

		// Wait for all images to load
		await expect(page.getByTestId('image-item')).toHaveCount(3);
	});

	test('should display position indicators', async ({ page }) => {
		const items = page.getByTestId('image-item');

		// Check position badges exist
		await expect(items.nth(0).locator('span').filter({ hasText: '1' })).toBeVisible();
		await expect(items.nth(1).locator('span').filter({ hasText: '2' })).toBeVisible();
		await expect(items.nth(2).locator('span').filter({ hasText: '3' })).toBeVisible();
	});

	test('should move image up', async ({ page }) => {
		const items = page.getByTestId('image-item');

		// Get the second image name (blue.png)
		const secondItemName = await items.nth(1).locator('p').textContent();
		expect(secondItemName).toContain('blue');

		// Click move up on second item
		await items.nth(1).hover();
		const moveUpButton = items.nth(1).getByRole('button', { name: 'Move up' });
		await moveUpButton.click();

		// Verify blue is now first
		const newFirstItemName = await items.nth(0).locator('p').textContent();
		expect(newFirstItemName).toContain('blue');
	});

	test('should move image down', async ({ page }) => {
		const items = page.getByTestId('image-item');

		// Get the first image name (red.png)
		const firstItemName = await items.nth(0).locator('p').textContent();
		expect(firstItemName).toContain('red');

		// Click move down on first item
		await items.nth(0).hover();
		const moveDownButton = items.nth(0).getByRole('button', { name: 'Move down' });
		await moveDownButton.click();

		// Verify red is now second
		const newSecondItemName = await items.nth(1).locator('p').textContent();
		expect(newSecondItemName).toContain('red');
	});

	test('should disable move up for first item', async ({ page }) => {
		const items = page.getByTestId('image-item');

		await items.nth(0).hover();
		const moveUpButton = items.nth(0).getByRole('button', { name: 'Move up' });
		await expect(moveUpButton).toBeDisabled();
	});

	test('should disable move down for last item', async ({ page }) => {
		const items = page.getByTestId('image-item');

		await items.nth(2).hover();
		const moveDownButton = items.nth(2).getByRole('button', { name: 'Move down' });
		await expect(moveDownButton).toBeDisabled();
	});

	test('should remove image', async ({ page }) => {
		const items = page.getByTestId('image-item');
		await expect(items).toHaveCount(3);

		// Remove the first item
		await items.nth(0).hover();
		const removeButton = items.nth(0).getByRole('button', { name: 'Remove' });
		await removeButton.click();

		// Verify count decreased
		await expect(items).toHaveCount(2);

		// Verify blue is now first
		const newFirstItemName = await items.nth(0).locator('p').textContent();
		expect(newFirstItemName).toContain('blue');
	});
});
