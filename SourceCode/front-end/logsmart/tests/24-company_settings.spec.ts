import { test, expect } from '@playwright/test';
import { register, clearMailhogEmails } from './utils';

let adminCreds: {
	email: string;
	password: string;
	companyName: string;
	firstName: string;
	lastName: string;
};

const PNG_BASE64 =
	'iVBORw0KGgoAAAANSUhEUgAAAEAAAABACAYAAACqaXHeAAAAIGNIUk0AAHomAACAhAAA+gAAAIDoAAB1MAAA6mAAADqYAAAXcJy6UTwAAAAGYktHRAD/AP8A/6C9p5MAAAtZSURBVHja7Zp5lFxFFcZ/t9573T2TbQiBZBJCFBA0IgRNkFUingMiED2iyBI2ARf2RYwIiAeIaEAMghA4BAIhCbuQsB3AoCzhCEQgIGGHbAxkIUyY6e29V9c/6s2kZ0hmuns6YdR85/T06+mq6nu/unWr7r0Fm7AJm7AJ/8eQWg5206yHKmqvvkdxyEDU98pq//Pdd+ldBNw4aw5pTRFKiGjbgIpnw7L6R+k0qwc1YM36xRAE3/MRERTl1D3H9A4Cps16ECuCb2OsSEpUvwrsDmwFpMsZwwY+2cEDUc902QyYGxPf5+EpwCl7jv5sCbhp1hxUAjxbBMyXgfOAvYEWYDUQlTN2FwQUkzEMMCp5/4Wi0wTRWlqCX00nTwGNUKP89ETZCcBcgU/KHUfa/hhAO3KDokAA3AJ8G5goSDiIQbeuYAVXPf1cTUio2AJumfEA6hZ8v0S4RuAY4PVyBtSSNnEqoLVxIIh0JsDBEGC5F8gBy4HvAefMltkzDtaDAXpMQsUWIArqNPgmbs0fBby+QxjyRhBw9BEHfarPjbMeAmupDzwKYbQFkLNGWt44ZD+GzZvfiJI1KjkrOkJhqUAuYUmSV4sY+bVaNcDl43ScLGPZrUMZ2mNLMJV2sEbJ90kB7Ae8ATwDrFf5qbc9DAgBMYUoGoNwpwoH2VTA8Cee2wy4FjjKio4A7hCYICqZThbhZfPZj0XkV8B9wKRhDBu/mMUAXPX0cxuPAIBMa9EHhgOLVLQlk0+vU/nps+ZgVAlsiBUzGrge6AP836Z8vHxhe1HdHWhWy3vAHOBUFT1HLBncDmABGZAZIIq2kTAbmDSCEeNPyY3uEQlVEYAzywBoQYVCurjuRgISx1gxYxLlI0F/ZpWXo7oMXiEcg9UiIvPFEEqslwLXAKep4ZdY6oEmoLEYF/ugoOjHYuQc4K/A7/9SN//InOYAuHre8xUrUtUuUIICwNFHHrjOL2MrGMNwRafgnObhFpnvGRj46jt+y1aD9xTVN61vFpswBkPOxDrRejIUOB94DXgIuA5hCrAQJVRVD7dNZoDJ9VKfF5W7LXajE9AtVMkhLAK+Dgwv+NHzmcgnv1n/LbF2Z7F674A3lrZ88rmhqAjq8RXQUcAC4DUjZqFVOxA4DBjJ2k1EgfeA7YAfqqf3ikrcywgQEF2JyumITgauzES+xOg9npGRYu0gieJ5+c0HYI1F1OyKsxYLciJiF6gqqnqDMWaGVdunM7+CTAbqqpVwgxKgFvx0SBwGS1TlDBG9EpjsIcsRRovVNX6u8EqcDkDNlqCTcDN7ImJf0EIdEkQEvkds4xzuPFBCr5D8r/4zI0C7+O7Y8Qcy7dYH8Rs+Jl7TfwlwOnAC7qj7IqqXe4XCsrBPBtACMB3hebAvbTEDYjl5r1HrHf/qpyt3ep1R7S5QNo4d/x2ijxsI/DwoS0aNnHthNmh8VtCHm97LTUEkRBWUZmAqyku2tQGM5dSxO21o8XpuAW0eadrMB5LP67IJSxhlQODFV/elniYUYch2fclu6YKh0iO0qV8DrGeGBdDaRYS19gEDgLE4p6Q9G2qdqreq8neQsgOujUZAMvMjgauBzaGKTblrGGC5wCGg83sdAcmELwA5EiVNtxagSQst6d4FnMvPAa/WTuaaEiAo2irwRHlBtpQkBCg7MFepoPHGJEBKhJKyBFw802GEuderfY0';

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

test.describe('Company Settings', () => {
	test('navigate_to_company_settings', async ({ page }) => {
		await page.getByRole('link', { name: 'Company Settings' }).click();
		await page.waitForURL('**/company-settings');

		await expect(page.getByRole('heading', { name: 'Company Settings' })).toBeVisible();
		await expect(page.getByText('Company Information')).toBeVisible();
	});

	test('update_company_details', async ({ page }) => {
		await page.getByRole('link', { name: 'Company Settings' }).click();
		await page.waitForURL('**/company-settings');

		const newName = `Updated ${adminCreds.companyName}`;
		const newAddress = '456 New Address, City, Country';

		await page.getByLabel('Company Name').fill(newName);
		await page.getByLabel('Company Headquarters Address').fill(newAddress);

		await page.getByRole('button', { name: 'Save Company Details' }).click();

		await expect(page.getByText('Company details saved successfully')).toBeVisible({
			timeout: 10000
		});
	});

	test('export_company_data', async ({ page }) => {
		await page.getByRole('link', { name: 'Company Settings' }).click();
		await page.waitForURL('**/company-settings');

		await page.getByRole('button', { name: /Export Company Data|Re-export Company Data/ }).click();

		await expect(page.getByText(/Data export initiated|You will receive an email/)).toBeVisible({
			timeout: 10000
		});
	});
});

test.describe('Company Data Export & Deletion', () => {
	test('delete_button_disabled_before_export', async ({ page }) => {
		await page.getByRole('link', { name: 'Company Settings' }).click();
		await page.waitForURL('**/company-settings');

		await page.reload();
		await page.waitForURL('**/company-settings');

		const deleteButton = page.getByRole('button', { name: 'Delete Company' });
		await expect(deleteButton).toBeDisabled();

		await expect(page.getByText('Export company data to enable deletion')).toBeVisible();
	});

	test('delete_button_enabled_after_export', async ({ page }) => {
		await page.getByRole('link', { name: 'Company Settings' }).click();
		await page.waitForURL('**/company-settings');

		await page.reload();
		await page.waitForURL('**/company-settings');

		await page.getByRole('button', { name: /Export Company Data|Re-export Company Data/ }).click();
		await page.waitForTimeout(2000);

		const deleteButton = page.getByRole('button', { name: 'Delete Company', exact: true });
		await expect(deleteButton).toBeEnabled({ timeout: 10000 });
	});
});

test.describe('Company Logo', () => {
	test('upload_and_persist_company_logo', async ({ page }) => {
		await page.getByRole('link', { name: 'Company Settings' }).click();
		await page.waitForURL('**/company-settings');

		const fileInput = page.locator('.file-input');
		await fileInput.setInputFiles({
			name: 'logo.png',
			mimeType: 'image/png',
			buffer: Buffer.from(PNG_BASE64, 'base64')
		});

		await page.waitForSelector('.cropper-modal', { timeout: 10000 });

		await page.getByRole('button', { name: 'Save', exact: true }).click();
		await page.waitForTimeout(3000);

		await expect(page.locator('img.picture-preview')).toBeVisible({ timeout: 10000 });

		await page.reload();
		await page.waitForURL('**/company-settings');
		await expect(page.locator('img.picture-preview')).toBeVisible();
	});

	test('delete_company_logo', async ({ page }) => {
		await page.getByRole('link', { name: 'Company Settings' }).click();
		await page.waitForURL('**/company-settings');

		const deleteButton = page.getByRole('button', { name: 'Delete', exact: true });
		if (await deleteButton.isVisible()) {
			await deleteButton.click();
			await page.waitForTimeout(1000);
		} else {
			const fileInput = page.locator('.file-input');
			await fileInput.setInputFiles({
				name: 'logo.png',
				mimeType: 'image/png',
				buffer: Buffer.from(PNG_BASE64, 'base64')
			});

			await page.waitForSelector('.cropper-modal', { timeout: 10000 });
			await page.getByRole('button', { name: 'Save', exact: true }).click();
			await page.waitForTimeout(3000);
		}

		const deleteBtn = page.getByRole('button', { name: 'Delete', exact: true });
		await expect(deleteBtn).toBeVisible({ timeout: 10000 });
		await deleteBtn.click();
		await page.waitForTimeout(2000);

		await expect(page.locator('img.picture-preview')).not.toBeVisible();
	});
});
