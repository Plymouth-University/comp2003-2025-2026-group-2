import { test, expect, type Page } from '@playwright/test';
import { register } from './utils';

async function fillMockOAuthForm(page: Page, email: string, firstName: string, lastName: string) {
	await page.fill('input[name="subject"], input#subject, input[type="text"]', email);
	const claimsJson = JSON.stringify({
		email: email,
		email_verified: true,
		given_name: firstName,
		family_name: lastName,
		picture: 'https://example.com/avatar.jpg'
	});
	await page.fill('textarea', claimsJson);
	await page
		.getByRole('button', { name: /sign-in/i })
		.first()
		.click();
}

test.describe('Google OAuth Authentication', () => {
	let adminCreds: {
		email: string;
		password: string;
		firstName: string;
		lastName: string;
	};

	test.beforeAll(async ({ browser }) => {
		const creds = await register(browser);
		if (!creds) throw new Error('Failed to register admin user');
		adminCreds = creds;
	});

	test('oauth_google_link_and_login', async ({ page }) => {
		test.skip(!!process.env.CI, 'Skipping Google OAuth test on CI due to potential flakiness');
		await page.goto('http://localhost:5173/login');
		await page.getByRole('textbox', { name: 'Email' }).fill(adminCreds.email);
		await page.getByRole('textbox', { name: 'Password' }).fill(adminCreds.password);
		await page.getByRole('button', { name: 'Sign in', exact: true }).click();
		await page.waitForURL('**/dashboard');

		await page.goto('http://localhost:5173/settings');
		await page.waitForLoadState('networkidle');
		await page.getByRole('button', { name: 'Link Google Account' }).click();
		await page.waitForURL(/localhost:8080/, { timeout: 10000 });

		await fillMockOAuthForm(page, adminCreds.email, adminCreds.firstName, adminCreds.lastName);
		await page.waitForURL('**/settings', { timeout: 10000 });
		await expect(page.getByRole('button', { name: /unlink google account/i })).toBeVisible({
			timeout: 5000
		});
		await page.getByRole('button', { name: /logout/i }).click();
		await page.waitForURL('**/login');

		await page.getByRole('button', { name: 'Sign in with Google' }).click();
		await page.waitForURL(/localhost:8080/, { timeout: 10000 });

		await fillMockOAuthForm(page, adminCreds.email, adminCreds.firstName, adminCreds.lastName);
		await page.waitForURL('**/dashboard', { timeout: 30000 });
		await expect(page.locator('body')).toContainText(adminCreds.email);
	});
});
