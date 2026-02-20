import { test, expect } from '@playwright/test';
import { register } from './utils';

test('create_and_list_branches', async ({ browser }) => {
	const adminCreds = await register(browser, false);
	if (!adminCreds) throw new Error('Failed to register admin user');

	// Navigate to Branches
	await adminCreds.page!.getByRole('link', { name: 'Branches' }).click();
	await adminCreds.page!.waitForURL('**/branches');

	await expect(adminCreds.page!.getByRole('heading', { name: 'BRANCH MANAGEMENT' })).toBeVisible();

	// Add Branch 1
	await adminCreds.page!.getByRole('textbox', { name: 'Branch Name' }).fill('London Office');
	await adminCreds.page!.getByRole('textbox', { name: 'Address' }).fill('123 Regent St, London');
	await adminCreds
		.page!.locator('form > div.search-container.relative.flex-1 > div > button')
		.first()
		.click();
	await adminCreds.page!.getByRole('button', { name: 'ADD BRANCH' }).click();

	await expect(adminCreds.page!.getByText('London Office')).toBeVisible();
	await expect(adminCreds.page!.getByText('123, Regent Street, London')).toBeVisible();

	// Add Branch 2
	await adminCreds.page!.getByRole('textbox', { name: 'Branch Name' }).fill('Manchester Hub');
	await adminCreds.page!.getByRole('textbox', { name: 'Address' }).fill('45 High St, Manchester');
	await adminCreds
		.page!.locator('form > div.search-container.relative.flex-1 > div > button')
		.first()
		.click();
	await adminCreds.page!.getByRole('button', { name: 'ADD BRANCH' }).click();

	await expect(adminCreds.page!.getByText('Manchester Hub')).toBeVisible();
	await expect(adminCreds.page!.getByText('45, High Street, Manchester')).toBeVisible();
	await expect(adminCreds.page!.getByText('London Office')).toBeVisible();

	await adminCreds.page!.close();
});
