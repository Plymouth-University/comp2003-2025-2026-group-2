import { test, expect } from '@playwright/test';

test('invite_user', async ({ page }) => {
	await page.goto('http://localhost:5173/');
	await page.getByRole('link', { name: 'Login' }).click();
	await page.waitForURL('**/login');
	await page.getByRole('textbox', { name: 'Email' }).click();
	await page.getByRole('textbox', { name: 'Email' }).fill('testuser@logsmart.app');
	await page.getByRole('textbox', { name: 'Email' }).press('Tab');
	await page.getByRole('textbox', { name: 'Password' }).fill('Test123!');
	await page.getByRole('button', { name: 'Sign in' }).click();
	await page.waitForURL('**/dashboard');
	await page.getByRole('link', { name: 'Users' }).click();
	await expect(page.locator('#eventHide')).toContainText('testuser@logsmart.app');
	await expect(page.locator('#eventHide')).toContainText('Test User');
	await expect(page.locator('#eventHide')).toContainText('Admin');
	await expect(page.locator('header')).toContainText('testuser@logsmart.app');
	await page.getByRole('button', { name: '➕' }).click();
	await page.getByRole('textbox', { name: "New user's email" }).click();
	await page.getByRole('textbox', { name: "New user's email" }).fill('testuser2@logsmart.app');
	await page.getByRole('button', { name: 'Send Invite' }).click();
});
