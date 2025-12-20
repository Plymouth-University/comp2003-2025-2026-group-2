import { test, expect } from '@playwright/test';

test('login_admin_valid', async ({ page }) => {
	await page.goto('http://localhost:5173/');
	await page.getByRole('link', { name: 'Login' }).click();
	await page.getByRole('textbox', { name: 'Email' }).click();
	await page.getByRole('textbox', { name: 'Email' }).fill('testuser@logsmart.app');
	await page.getByRole('textbox', { name: 'Email' }).press('Tab');
	await page.getByRole('textbox', { name: 'Password' }).fill('Test123!');
	await page.getByRole('button', { name: 'Sign in' }).click();
	await page.waitForURL('**/dashboard');
	await expect(page.locator('span')).toContainText('testuser@logsmart.app');
	await expect(page.locator('body')).toContainText('Test User');
	await expect(page.locator('body')).toContainText('testuser@logsmart.app');
	await expect(page.locator('body')).toContainText('TestCompany1');
	await expect(page.locator('body')).toContainText('admin');
});

test('login_admin_invalid_empty_email', async ({ page }) => {
	await page.goto('http://localhost:5173/');
	await page.getByRole('link', { name: 'Login' }).click();
	await page.getByRole('textbox', { name: 'Email' }).click();
	await page.getByRole('textbox', { name: 'Email' }).press('Tab');
	await page.getByRole('textbox', { name: 'Password' }).fill('Test123!');
	await expect(page.getByRole('button', { name: 'Sign in' })).toBeDisabled();
	await expect(page.locator('body')).toContainText('Enter a valid email address');
	await expect(page).toHaveURL('/login');
});

test('login_admin_invalid_empty_password', async ({ page }) => {
	await page.goto('http://localhost:5173/');
	await page.getByRole('link', { name: 'Login' }).click();
	await page.getByRole('textbox', { name: 'Email' }).fill('testuser@logsmart.app');
	await page.getByRole('textbox', { name: 'Email' }).press('Tab');
	await expect(page.getByRole('button', { name: 'Sign in' })).toBeDisabled();
	await expect(page).toHaveURL('/login');
});

test('login_admin_invalid_both_empty', async ({ page }) => {
	await page.goto('http://localhost:5173/');
	await page.getByRole('link', { name: 'Login' }).click();
	await expect(page.getByRole('button', { name: 'Sign in' })).toBeDisabled();
	await expect(page).toHaveURL('/login');
});

test('login_admin_invalid_email_format', async ({ page }) => {
	await page.goto('http://localhost:5173/');
	await page.getByRole('link', { name: 'Login' }).click();
	await page.getByRole('textbox', { name: 'Email' }).fill('not-an-email');
	await page.getByRole('textbox', { name: 'Email' }).press('Tab');
	await page.getByRole('textbox', { name: 'Password' }).fill('Test123!');
	await expect(page.getByRole('button', { name: 'Sign in' })).toBeDisabled();
	await expect(page).toHaveURL('/login');
});

test('login_admin_invalid_wrong_password', async ({ page }) => {
	await page.goto('http://localhost:5173/');
	await page.getByRole('link', { name: 'Login' }).click();
	await page.getByRole('textbox', { name: 'Email' }).fill('testuser@logsmart.app');
	await page.getByRole('textbox', { name: 'Email' }).press('Tab');
	await page.getByRole('textbox', { name: 'Password' }).fill('WrongPassword123!');
	await page.getByRole('button', { name: 'Sign in' }).click();
	await expect(page.locator('body')).toContainText('Invalid email or password');
	await expect(page).toHaveURL('/login');
});

test('login_admin_invalid_nonexistent_email', async ({ page }) => {
	await page.goto('http://localhost:5173/');
	await page.getByRole('link', { name: 'Login' }).click();
	await page.getByRole('textbox', { name: 'Email' }).fill('nonexistent@logsmart.app');
	await page.getByRole('textbox', { name: 'Email' }).press('Tab');
	await page.getByRole('textbox', { name: 'Password' }).fill('Test123!');
	await page.getByRole('button', { name: 'Sign in' }).click();
	await expect(page.locator('body')).toContainText('Invalid email or password');
	await expect(page).toHaveURL('/login');
});
