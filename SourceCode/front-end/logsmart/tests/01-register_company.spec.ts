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
	if (
		await page
			.locator('body')
			.textContent()
			.then((text) => text?.includes('Email already exists'))
	) {
		console.error('Email already exists. Cannot register the same email again.');
		return;
	}
	await page.waitForURL('**/dashboard');
	await expect(page.locator('span')).toContainText('testuser@logsmart.app');
	await expect(page.locator('body')).toContainText('Test User');
	await expect(page.locator('body')).toContainText('testuser@logsmart.app');
	await expect(page.locator('body')).toContainText('TestCompany1');
	await expect(page.locator('body')).toContainText('admin');
});

test('register_company_invalid_empty_company_name', async ({ page }) => {
	await page.goto('http://localhost:5173/');
	await page.getByRole('link', { name: 'Register Company' }).click();
	await page.waitForURL('**/register-company');
	await page.getByRole('textbox', { name: 'Company Name' }).press('Tab');
	await page.getByRole('textbox', { name: 'Company Address' }).fill('TestAddress1');
	await expect(page.getByRole('button', { name: 'Next Step' })).toBeDisabled();
	await expect(page).toHaveURL('/register-company');
});

test('register_company_invalid_empty_address', async ({ page }) => {
	await page.goto('http://localhost:5173/');
	await page.getByRole('link', { name: 'Register Company' }).click();
	await page.waitForURL('**/register-company');
	await page.getByRole('textbox', { name: 'Company Name' }).fill('TestCompany2');
	await page.getByRole('textbox', { name: 'Company Address' }).press('Tab');
	await expect(page.getByRole('button', { name: 'Next Step' })).toBeDisabled();
	await expect(page).toHaveURL('/register-company');
});

test('register_company_invalid_empty_first_name', async ({ page }) => {
	await page.goto('http://localhost:5173/');
	await page.getByRole('link', { name: 'Register Company' }).click();
	await page.waitForURL('**/register-company');
	await page.getByRole('textbox', { name: 'Company Name' }).fill('TestCompany3');
	await page.getByRole('textbox', { name: 'Company Address' }).fill('TestAddress3');
	await page.getByRole('button', { name: 'Next Step' }).click();
	await page.getByRole('textbox', { name: 'First Name' }).press('Tab');
	await page.getByRole('textbox', { name: 'Last Name' }).fill('User');
	await page.getByRole('textbox', { name: 'Email' }).fill('test3@logsmart.app');
	await page.getByRole('textbox', { name: 'Password Show password', exact: true }).fill('Test123!');
	await page.getByRole('textbox', { name: 'Confirm Password Show password' }).fill('Test123!');
	await expect(page.getByRole('button', { name: 'Create Account' })).toBeDisabled();
});

test('register_company_invalid_empty_last_name', async ({ page }) => {
	await page.goto('http://localhost:5173/');
	await page.getByRole('link', { name: 'Register Company' }).click();
	await page.waitForURL('**/register-company');
	await page.getByRole('textbox', { name: 'Company Name' }).fill('TestCompany4');
	await page.getByRole('textbox', { name: 'Company Address' }).fill('TestAddress4');
	await page.getByRole('button', { name: 'Next Step' }).click();
	await page.getByRole('textbox', { name: 'First Name' }).fill('Test');
	await page.getByRole('textbox', { name: 'Last Name' }).press('Tab');
	await page.getByRole('textbox', { name: 'Email' }).fill('test4@logsmart.app');
	await page.getByRole('textbox', { name: 'Password Show password', exact: true }).fill('Test123!');
	await page.getByRole('textbox', { name: 'Confirm Password Show password' }).fill('Test123!');
	await expect(page.getByRole('button', { name: 'Create Account' })).toBeDisabled();
});

test('register_company_invalid_empty_email', async ({ page }) => {
	await page.goto('http://localhost:5173/');
	await page.getByRole('link', { name: 'Register Company' }).click();
	await page.waitForURL('**/register-company');
	await page.getByRole('textbox', { name: 'Company Name' }).fill('TestCompany5');
	await page.getByRole('textbox', { name: 'Company Address' }).fill('TestAddress5');
	await page.getByRole('button', { name: 'Next Step' }).click();
	await page.getByRole('textbox', { name: 'First Name' }).fill('Test');
	await page.getByRole('textbox', { name: 'Last Name' }).fill('User');
	await page.getByRole('textbox', { name: 'Email' }).press('Tab');
	await page.getByRole('textbox', { name: 'Password Show password', exact: true }).fill('Test123!');
	await page.getByRole('textbox', { name: 'Confirm Password Show password' }).fill('Test123!');
	await expect(page.getByRole('button', { name: 'Create Account' })).toBeDisabled();
});

test('register_company_invalid_email_format', async ({ page }) => {
	await page.goto('http://localhost:5173/');
	await page.getByRole('link', { name: 'Register Company' }).click();
	await page.waitForURL('**/register-company');
	await page.getByRole('textbox', { name: 'Company Name' }).fill('TestCompany6');
	await page.getByRole('textbox', { name: 'Company Address' }).fill('TestAddress6');
	await page.getByRole('button', { name: 'Next Step' }).click();
	await page.getByRole('textbox', { name: 'First Name' }).fill('Test');
	await page.getByRole('textbox', { name: 'Last Name' }).fill('User');
	await page.getByRole('textbox', { name: 'Email' }).fill('not-an-email');
	await page.getByRole('textbox', { name: 'Password Show password', exact: true }).fill('Test123!');
	await page.getByRole('textbox', { name: 'Confirm Password Show password' }).fill('Test123!');
	await expect(page.getByRole('button', { name: 'Create Account' })).toBeDisabled();
});

test('register_company_invalid_empty_password', async ({ page }) => {
	await page.goto('http://localhost:5173/');
	await page.getByRole('link', { name: 'Register Company' }).click();
	await page.waitForURL('**/register-company');
	await page.getByRole('textbox', { name: 'Company Name' }).fill('TestCompany7');
	await page.getByRole('textbox', { name: 'Company Address' }).fill('TestAddress7');
	await page.getByRole('button', { name: 'Next Step' }).click();
	await page.getByRole('textbox', { name: 'First Name' }).fill('Test');
	await page.getByRole('textbox', { name: 'Last Name' }).fill('User');
	await page.getByRole('textbox', { name: 'Email' }).fill('test7@logsmart.app');
	await page.getByRole('textbox', { name: 'Password Show password', exact: true }).press('Tab');
	await page.getByRole('textbox', { name: 'Confirm Password Show password' }).fill('Test123!');
	await expect(page.getByRole('button', { name: 'Create Account' })).toBeDisabled();
});

test('register_company_invalid_empty_confirm_password', async ({ page }) => {
	await page.goto('http://localhost:5173/');
	await page.getByRole('link', { name: 'Register Company' }).click();
	await page.waitForURL('**/register-company');
	await page.getByRole('textbox', { name: 'Company Name' }).fill('TestCompany8');
	await page.getByRole('textbox', { name: 'Company Address' }).fill('TestAddress8');
	await page.getByRole('button', { name: 'Next Step' }).click();
	await page.getByRole('textbox', { name: 'First Name' }).fill('Test');
	await page.getByRole('textbox', { name: 'Last Name' }).fill('User');
	await page.getByRole('textbox', { name: 'Email' }).fill('test8@logsmart.app');
	await page.getByRole('textbox', { name: 'Password Show password', exact: true }).fill('Test123!');
	await page.getByRole('textbox', { name: 'Confirm Password Show password' }).press('Tab');
	await expect(page.getByRole('button', { name: 'Create Account' })).toBeDisabled();
});

test('register_company_invalid_password_mismatch', async ({ page }) => {
	await page.goto('http://localhost:5173/');
	await page.getByRole('link', { name: 'Register Company' }).click();
	await page.waitForURL('**/register-company');
	await page.getByRole('textbox', { name: 'Company Name' }).fill('TestCompany9');
	await page.getByRole('textbox', { name: 'Company Address' }).fill('TestAddress9');
	await page.getByRole('button', { name: 'Next Step' }).click();
	await page.getByRole('textbox', { name: 'First Name' }).fill('Test');
	await page.getByRole('textbox', { name: 'Last Name' }).fill('User');
	await page.getByRole('textbox', { name: 'Email' }).fill('test9@logsmart.app');
	await page.getByRole('textbox', { name: 'Password Show password', exact: true }).fill('Test123!');
	await page.getByRole('textbox', { name: 'Confirm Password Show password' }).fill('Different456!');
	await expect(page.getByRole('button', { name: 'Create Account' })).toBeDisabled();
});

test('register_company_invalid_duplicate_email', async ({ page }) => {
	await page.goto('http://localhost:5173/');
	await page.getByRole('link', { name: 'Register Company' }).click();
	await page.waitForURL('**/register-company');
	await page.getByRole('textbox', { name: 'Company Name' }).fill('TestCompany10');
	await page.getByRole('textbox', { name: 'Company Address' }).fill('TestAddress10');
	await page.getByRole('button', { name: 'Next Step' }).click();
	await page.getByRole('textbox', { name: 'First Name' }).fill('Test');
	await page.getByRole('textbox', { name: 'Last Name' }).fill('User');
	await page.getByRole('textbox', { name: 'Email' }).fill('testuser@logsmart.app');
	await page.getByRole('textbox', { name: 'Password Show password', exact: true }).fill('Test123!');
	await page.getByRole('textbox', { name: 'Confirm Password Show password' }).fill('Test123!');
	await page.getByRole('button', { name: 'Create Account' }).click();
	await expect(page.locator('body')).toContainText('Email already exists');
	await expect(page).toHaveURL('/register-company');
});
