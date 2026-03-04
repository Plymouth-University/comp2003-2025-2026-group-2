import { test, expect } from '@playwright/test';
import { register } from './utils';

let adminCreds: { email: string; password: string };

const PNG_BASE64 =
	'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';

test.beforeAll(async ({ browser }) => {
	const creds = await register(browser);
	if (!creds) throw new Error('Failed to register admin user');
	adminCreds = { email: creds.email, password: creds.password };
});

test.beforeEach(async ({ page }) => {
	await page.goto('http://localhost:5173/');
	await page.getByRole('link', { name: 'Login' }).click();
	await page.getByRole('textbox', { name: 'Email' }).fill(adminCreds.email);
	await page.getByRole('textbox', { name: 'Password' }).fill(adminCreds.password);
	await page.getByRole('button', { name: 'Sign in', exact: true }).click();
	await page.waitForURL('**/dashboard');
});

test.describe('Profile Picture', () => {
	test('upload_and_persist_profile_picture', async ({ page }) => {
		await page.getByRole('link', { name: 'Settings' }).click();
		await page.waitForURL('**/settings');

		const fileInput = page.locator('.file-input');
		await fileInput.setInputFiles({
			name: 'avatar.png',
			mimeType: 'image/png',
			buffer: Buffer.from(PNG_BASE64, 'base64')
		});

		await expect(page.getByRole('button', { name: 'Save', exact: true })).toBeVisible();
		const uploadResponse = page.waitForResponse(
			(resp) => resp.url().includes('/api/auth/profile-picture') && resp.status() === 200
		);
		await page.getByRole('button', { name: 'Save', exact: true }).click();
		await uploadResponse;

		await expect(page.locator('img.profile-preview')).toBeVisible();

		await page.reload();
		await page.waitForURL('**/settings');
		await expect(page.locator('img.profile-preview')).toBeVisible();
	});
});
