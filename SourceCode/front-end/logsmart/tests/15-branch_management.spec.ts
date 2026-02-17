import { test, expect } from '@playwright/test';
import { register } from './utils';

test('create_and_list_branches', async ({ browser }) => {
	const adminCreds = await register(browser);
	if (!adminCreds) throw new Error('Failed to register admin user');

	const page = await browser.newPage();
	await page.goto('http://localhost:5173/login');
	await page.getByRole('textbox', { name: 'Email' }).fill(adminCreds.email);
	await page.getByRole('textbox', { name: 'Password' }).fill(adminCreds.password);
	await page.getByRole('button', { name: 'Sign in', exact: true }).click();
	await page.waitForURL('**/dashboard');

	// Navigate to Branches
	await page.getByRole('link', { name: 'Branches' }).click();
	await page.waitForURL('**/branches');

	await expect(page.getByRole('heading', { name: 'BRANCH MANAGEMENT' })).toBeVisible();

	// Add Branch 1
	await page.getByRole('textbox', { name: 'Branch Name' }).fill('London Office');
	await page.getByRole('textbox', { name: 'Address' }).fill('123 Regent St, London');
	await page.locator('form > div.search-container.relative.flex-1 > div > button').first().click();
	await page.getByRole('button', { name: 'ADD BRANCH' }).click();

	await expect(page.getByText('London Office')).toBeVisible();
	await expect(page.getByText('123, Regent Street, London')).toBeVisible();

	// Add Branch 2
	await page.getByRole('textbox', { name: 'Branch Name' }).fill('Manchester Hub');
	await page.getByRole('textbox', { name: 'Address' }).fill('45 High St, Manchester');
	await page.locator('form > div.search-container.relative.flex-1 > div > button').first().click();
	await page.getByRole('button', { name: 'ADD BRANCH' }).click();

	await expect(page.getByText('Manchester Hub')).toBeVisible();
	await expect(page.getByText('45, High Street, Manchester')).toBeVisible();
	await expect(page.getByText('London Office')).toBeVisible();

	await page.close();
});
