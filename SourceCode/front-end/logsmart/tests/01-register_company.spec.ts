import { test, expect } from '@playwright/test';
import { register } from './utils';
import { validatePasswordField } from './shared-validators';

test('register_company', async ({ browser }) => {
	const result = await register(browser, false);
	if (!result || !result.page) throw new Error('Registration failed');
	const { page, companyName, firstName, lastName, email } = result;
	await page.goto('http://localhost:5173/dashboard');
	await expect(page.locator('span.text-sm')).toContainText(email);
	await expect(page.locator('body')).toContainText(`${firstName} ${lastName}`);
	await expect(page.locator('body')).toContainText(email);
	await expect(page.locator('body')).toContainText(companyName);
	await expect(page.locator('body')).toContainText('Company Manager');
	await page.close();
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

test('register_company_validate_password_requirements', async ({ page }) => {
	await page.goto('http://localhost:5173/');
	await page.getByRole('link', { name: 'Register Company' }).click();
	await page.waitForURL('**/register-company');
	await page.getByRole('textbox', { name: 'Company Name' }).fill('TestCompany_pwd');
	await page.getByRole('textbox', { name: 'Company Address' }).fill('TestAddress_pwd');
	await page.getByRole('button', { name: 'Next Step' }).click();
	await page.getByRole('textbox', { name: 'First Name' }).fill('Test');
	await page.getByRole('textbox', { name: 'Last Name' }).fill('User');
	await page.getByRole('textbox', { name: 'Email' }).fill('test_pwd@logsmart.app');

	// Test 1: Empty password validation (consolidated from register_company_invalid_empty_password)
	await validatePasswordField(page, 'empty', 'registration');

	// Fill valid password and confirm for next test
	await page.getByTestId('password-input').fill('Test123!');

	// Test 2: Mismatch validation (consolidated from register_company_invalid_password_mismatch)
	await page.getByTestId('confirm-password-input').fill('Different456!');
	await expect(page.getByRole('button', { name: 'Create Account' })).toBeDisabled();

	// Test 3: Weak password validation
	await page.getByTestId('password-input').fill('weak');
	await page.getByTestId('confirm-password-input').fill('weak');
	await validatePasswordField(page, 'weak', 'registration');
});

test('register_company_invalid_duplicate_email', async ({ page, browser }) => {
	const result = await register(browser, true);
	if (!result) throw new Error('Initial registration failed');
	await page.goto('http://localhost:5173/');
	await page.getByRole('link', { name: 'Register Company' }).click();
	await page.waitForURL('**/register-company');
	await page.getByRole('textbox', { name: 'Company Name' }).fill('TestCompany10');
	await page.getByRole('textbox', { name: 'Company Address' }).fill('TestAddress10');
	await page.getByRole('button', { name: 'Next Step' }).click();
	await page.getByRole('textbox', { name: 'First Name' }).fill('Test');
	await page.getByRole('textbox', { name: 'Last Name' }).fill('User');
	await page.getByRole('textbox', { name: 'Email' }).fill(result.email);
	await page.getByTestId('password-input').fill('Test123!');
	await page.getByTestId('confirm-password-input').fill('Test123!');
	await page.getByRole('button', { name: 'Create Account' }).click();
	await expect(page.locator('body')).toContainText('Email already exists');
	await expect(page).toHaveURL('/register-company');
});
