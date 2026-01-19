import { defineConfig, devices } from '@playwright/test';
import dotenv from 'dotenv';

dotenv.config();

const basePath = process.env.BASE_PATH || '';

export default defineConfig({
	testDir: './tests',
	fullyParallel: true,
	forbidOnly: !!process.env.CI,
	retries: process.env.CI ? 2 : 0,
	// Limit workers to prevent resource contention with test server
	workers: process.env.CI ? 1 : 4,
	reporter: 'html',
	use: {
		baseURL: `http://localhost:4173${basePath}`,
		trace: 'on-first-retry'
	},
	projects: [
		{
			name: 'chromium',
			use: { ...devices['Desktop Chrome'] }
		},
		{
			name: 'firefox',
			use: { ...devices['Desktop Firefox'] }
		},
		{
			name: 'webkit',
			use: { ...devices['Desktop Safari'] }
		}
	],
	webServer: {
		command: 'npm run build && npm run preview',
		url: `http://localhost:4173${basePath}`,
		reuseExistingServer: !process.env.CI,
		timeout: 120000
	}
});
