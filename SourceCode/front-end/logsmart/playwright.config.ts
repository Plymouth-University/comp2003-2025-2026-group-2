/// <reference types="node" />
import { defineConfig, devices } from '@playwright/test';

const COOKIE_BANNER_DISMISSED_STATE = {
	cookies: [
		{
			name: 'cookies_notice_dismissed',
			value: 'true',
			domain: 'localhost',
			path: '/',
			expires: Math.floor(Date.now() / 1000) + 60 * 60 * 24 * 365,
			httpOnly: false,
			secure: false,
			sameSite: 'Lax' as const
		}
	],
	origins: []
};

export default defineConfig({
	testDir: './tests',
	fullyParallel: true,
	forbidOnly: !!process.env.CI,
	retries: process.env.CI ? 2 : 1,
	workers: process.env.CI ? 4 : 6,
	reporter: 'list',
	use: {
		baseURL: process.env.FRONTEND_URL || 'http://localhost:5173',
		trace: 'on-first-retry',
		storageState: COOKIE_BANNER_DISMISSED_STATE
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
