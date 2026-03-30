import { test, expect } from '@playwright/test';
import { register, clearMailhogEmails } from './utils';

let adminCreds: {
	email: string;
	password: string;
	companyName: string;
	firstName: string;
	lastName: string;
};

test.beforeAll(async ({ browser }) => {
	await clearMailhogEmails();
	const creds = await register(browser);
	if (!creds) throw new Error('Failed to register admin user');
	adminCreds = creds;
});

test.beforeEach(async ({ page }) => {
	await page.goto('http://localhost:5173/');
	await page.getByRole('link', { name: 'Login' }).click();
	await page.getByRole('textbox', { name: 'Email' }).fill(adminCreds.email);
	await page.getByRole('textbox', { name: 'Password' }).fill(adminCreds.password);
	await page.getByRole('button', { name: 'Sign in', exact: true }).click();
	await page.waitForURL('**/dashboard');
});

test.describe('Confirm Company Deletion Page', () => {
	test('shows_error_when_missing_params', async ({ page }) => {
		await page.goto('http://localhost:5173/confirm-company-deletion');
		await expect(page.getByText('Missing company ID or confirmation token')).toBeVisible({ timeout: 10000 });
	});

	test('shows_error_when_invalid_token', async ({ page }) => {
		await page.goto(
			'http://localhost:5173/confirm-company-deletion?company_id=test-123&token=invalid-token'
		);
		await expect(
			page.getByText(/Failed to delete company|The token may have expired|Invalid or expired/)
		).toBeVisible({ timeout: 10000 });
	});
});
