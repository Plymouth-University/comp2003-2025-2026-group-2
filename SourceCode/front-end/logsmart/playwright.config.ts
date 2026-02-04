/// <reference types="node" />
import { defineConfig, devices } from '@playwright/test';

export default defineConfig({
	testDir: './tests',
	fullyParallel: true,
	forbidOnly: !!process.env.CI,
	retries: process.env.CI ? 2 : 1,
	workers: process.env.CI ? 4 : 6,
	reporter: 'list',
	use: {
		baseURL: process.env.FRONTEND_URL || 'http://localhost:5173',
		trace: 'on-first-retry'
	},

	projects: [
		{
			name: 'chromium',
			use: { ...devices['chromium'] }
		}
	],

	globalSetup: 'tests/global-setup.ts',
	globalTeardown: 'tests/global-teardown.ts'
});
