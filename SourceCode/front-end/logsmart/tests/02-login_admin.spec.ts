import { test, expect } from '@playwright/test';
import { register } from './utils';

let adminCreds: {
	email: string;
	password: string;
	companyName: string;
	firstName: string;
	lastName: string;
};

test.beforeAll(async ({ browser }) => {
	const creds = await register(browser);
	if (!creds) throw new Error('Failed to register admin user');
	adminCreds = creds;
});

test('login_admin_valid', async ({ page }) => {
	await page.goto('http://localhost:5173/');
	await page.getByRole('link', { name: 'Login' }).click();
	await page.getByRole('textbox', { name: 'Email' }).click();
	await page.getByRole('textbox', { name: 'Email' }).fill(adminCreds.email);
	await page.getByRole('textbox', { name: 'Email' }).press('Tab');
	await page.getByRole('textbox', { name: 'Password' }).fill(adminCreds.password);
	await page.getByRole('button', { name: 'Sign in' }).click();
	await page.waitForURL('**/dashboard');
	await expect(page.locator('span')).toContainText(adminCreds.email);
	await expect(page.locator('body')).toContainText(adminCreds.firstName.split('-')[0]);
	await expect(page.locator('body')).toContainText(adminCreds.email);
	await expect(page.locator('body')).toContainText(adminCreds.companyName);
	await expect(page.locator('body')).toContainText('Company Admin');
});

test('login_admin_invalid_empty_email', async ({ page }) => {
	await page.goto('http://localhost:5173/');
	await page.getByRole('link', { name: 'Login' }).click();
	await page.getByRole('textbox', { name: 'Email' }).click();
	await page.getByRole('textbox', { name: 'Email' }).press('Tab');
	await page.getByRole('textbox', { name: 'Password' }).fill('Test123!');
	await expect(page.getByRole('button', { name: 'Sign in' })).toBeDisabled();
	await expect(page).toHaveURL('/login');
});

test('login_admin_invalid_empty_password', async ({ page }) => {
	await page.goto('http://localhost:5173/');
	await page.getByRole('link', { name: 'Login' }).click();
	await page.getByRole('textbox', { name: 'Email' }).fill(adminCreds.email);
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
	await page.getByRole('textbox', { name: 'Email' }).fill(adminCreds.email);
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

test('logout_admin', async ({ page }) => {
	await page.goto('http://localhost:5173/');
	await page.getByRole('link', { name: 'Login' }).click();
	await page.getByRole('textbox', { name: 'Email' }).fill(adminCreds.email);
	await page.getByRole('textbox', { name: 'Password' }).fill(adminCreds.password);
	await page.getByRole('button', { name: 'Sign in' }).click();
	await page.waitForURL('**/dashboard');
	await page.getByRole('button', { name: 'Logout' }).click();
	await page.waitForURL('**/login');
	await expect(page).toHaveURL('/login');
});

test('logout_redirects_to_login_on_protected_route_access', async ({ page }) => {
	await page.goto('http://localhost:5173/');
	await page.getByRole('link', { name: 'Login' }).click();
	await page.getByRole('textbox', { name: 'Email' }).fill(adminCreds.email);
	await page.getByRole('textbox', { name: 'Password' }).fill(adminCreds.password);
	await page.getByRole('button', { name: 'Sign in' }).click();
	await page.waitForURL('**/dashboard');
	await page.getByRole('button', { name: 'Logout' }).click();
	await page.waitForURL('**/login');
	await page.goto('http://localhost:5173/dashboard');
	await page.waitForURL('**/login');
	await expect(page).toHaveURL('/login');
});

test('unauthenticated_access_to_dashboard_redirects', async ({ page }) => {
	await page.goto('http://localhost:5173/dashboard');
	await page.waitForURL('**/login');
	await expect(page).toHaveURL('/login');
});

test('unauthenticated_access_to_users_admin_redirects', async ({ page }) => {
	await page.goto('http://localhost:5173/users-admin');
	await page.waitForURL('**/login');
	await expect(page).toHaveURL('/login');
});

test('unauthenticated_access_to_templates_dashboard_redirects', async ({ page }) => {
	await page.goto('http://localhost:5173/templates-dashboard');
	await page.waitForURL('**/login');
	await expect(page).toHaveURL('/login');
});
