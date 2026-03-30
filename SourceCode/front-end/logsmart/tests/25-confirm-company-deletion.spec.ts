import { test, expect } from '@playwright/test';
import { register, clearMailhogEmails, getCompanyDeletionToken } from './utils';

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
		await expect(page.getByText('Missing company ID or confirmation token')).toBeVisible({
			timeout: 10000
		});
	});

	test('shows_error_when_invalid_token', async ({ page }) => {
		await page.goto(
			'http://localhost:5173/confirm-company-deletion?company_id=test-123&token=invalid-token'
		);
		await expect(
			page.getByText(
				/Failed to delete company|The token may have expired|Invalid or expired|Company not found/
			)
		).toBeVisible({ timeout: 10000 });
	});
});

test.describe('Complete Company Deletion Flow', () => {
	test('full_deletion_flow_with_email_confirmation', async ({ browser }) => {
		await clearMailhogEmails();
		const creds = await register(browser);
		if (!creds) throw new Error('Failed to register admin user');

		const page = await browser.newPage();
		await page.goto('http://localhost:5173/login');
		await page.getByRole('textbox', { name: 'Email' }).fill(creds.email);
		await page.getByRole('textbox', { name: 'Password' }).fill(creds.password);
		await page.getByRole('button', { name: 'Sign in', exact: true }).click();
		await page.waitForURL('**/dashboard');

		await page.getByRole('link', { name: 'Company Settings' }).click();
		await page.waitForURL('**/company-settings');
		await page.reload();
		await page.waitForURL('**/company-settings');

		await page.getByRole('button', { name: /Export Company Data|Re-export Company Data/ }).click();
		await page.waitForTimeout(2000);

		const deleteButton = page.getByRole('button', { name: 'Delete Company' });
		await expect(deleteButton).toBeEnabled({ timeout: 10000 });

		page.on('dialog', async (dialog) => {
			await dialog.accept();
		});

		await deleteButton.click();
		await page.waitForTimeout(2000);

		const tokenData = await getCompanyDeletionToken(creds.email);
		expect(tokenData).toBeTruthy();

		const { token, companyId } = JSON.parse(tokenData!);

		const confirmPage = await browser.newPage();
		await confirmPage.goto(
			`http://localhost:5173/confirm-company-deletion?company_id=${companyId}&token=${token}`
		);
		await confirmPage.waitForURL('**/confirm-company-deletion**');

		await expect(confirmPage.locator('body')).toContainText('Confirm Company Deletion');

		await expect(
			confirmPage.getByRole('textbox', { name: 'Type company name to confirm' })
		).toBeVisible({ timeout: 10000 });
		await confirmPage
			.getByRole('textbox', { name: 'Type company name to confirm' })
			.fill(creds.companyName);

		await confirmPage.getByRole('button', { name: 'Delete Company' }).click();

		await expect(confirmPage.locator('body')).toContainText('Success');

		await page.close();
		await confirmPage.close();
	});
});
