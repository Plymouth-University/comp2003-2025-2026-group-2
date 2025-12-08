import { test, expect } from '@playwright/test';

test('register_company', async ({ page }) => {
	await page.goto('http://localhost:5173/');
	await page.getByRole('link', { name: 'Register Company' }).click();
	await page.waitForURL('**/register-company');
	await page.getByRole('textbox', { name: 'Company Name' }).click();
	await page.getByRole('textbox', { name: 'Company Name' }).fill('TestCompany1');
	await page.getByRole('textbox', { name: 'Company Name' }).press('Tab');
	await page
		.getByRole('textbox', { name: 'Company Address' })
		.fill('TestAddress1, ABC\nSecond Line,\n2!');
	await page.getByRole('button', { name: 'Next Step' }).click();
	await page.getByRole('textbox', { name: 'First Name' }).click();
	await page.getByRole('textbox', { name: 'First Name' }).fill('Test');
	await page.getByRole('textbox', { name: 'First Name' }).press('Tab');
	await page.getByRole('textbox', { name: 'Last Name' }).fill('User');
	await page.getByRole('textbox', { name: 'Last Name' }).press('Tab');
	await page.getByRole('textbox', { name: 'Email' }).fill('testuser@logsmart.app');
	await page.getByRole('textbox', { name: 'Email' }).press('Tab');
	await page.getByRole('textbox', { name: 'Password Show password', exact: true }).fill('Test123!');
	await page.getByRole('textbox', { name: 'Confirm Password Show password' }).click();
	await page.getByRole('textbox', { name: 'Confirm Password Show password' }).fill('Test123!');
	await page.getByRole('button', { name: 'Create Account' }).click();
	await page.waitForURL('**/dashboard');
	await expect(page.locator('span')).toContainText('testuser@logsmart.app');
	await expect(page.locator('body')).toContainText('Test User');
	await expect(page.locator('body')).toContainText('testuser@logsmart.app');
	await expect(page.locator('body')).toContainText('TestCompany1');
	await expect(page.locator('body')).toContainText('admin');
});
