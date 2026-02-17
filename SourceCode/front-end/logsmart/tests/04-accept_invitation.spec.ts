import { test, expect } from '@playwright/test';
import { register, sendInvitation, acceptInvitation, createBranch } from './utils';

let adminCreds: {
	email: string;
	password: string;
	companyName: string;
	firstName: string;
	lastName: string;
};

const BRANCH_NAME = 'Test Branch';

test.beforeAll(async ({ browser }) => {
	const creds = await register(browser);
	if (!creds) throw new Error('Failed to register admin user');
	adminCreds = creds;

	const page = await browser.newPage();
	await page.goto('http://localhost:5173/login');
	await page.getByRole('textbox', { name: 'Email' }).fill(adminCreds.email);
	await page.getByRole('textbox', { name: 'Password' }).fill(adminCreds.password);
	await page.getByRole('button', { name: 'Sign in', exact: true }).click();
	await page.waitForURL('**/dashboard');

	await createBranch(page, BRANCH_NAME, '123 Test St');
	await page.close();
});

test('accept_invitation_valid', async ({ page, browser }) => {
	const inviteeEmail = `invited-${Date.now()}@logsmart.app`;
	const invitationToken = await sendInvitation(
		browser,
		adminCreds,
		inviteeEmail,
		'staff',
		BRANCH_NAME
	);
	expect(invitationToken).toBeTruthy();

	await page.goto(`http://localhost:5173/accept-invitation?token=${invitationToken}`);
	await page.waitForURL('**/accept-invitation**');
	await expect(page.locator('body')).toContainText(adminCreds.companyName);

	const success = await acceptInvitation(
		page,
		invitationToken!,
		'Invited',
		'User',
		'Invited123!',
		'**/logs-list'
	);
	expect(success).toBe(true);
});

test('accept_invitation_invalid_empty_first_name', async ({ page, browser }) => {
	const token = await sendInvitation(
		browser,
		adminCreds,
		'validation1@logsmart.app',
		'staff',
		BRANCH_NAME
	);
	expect(token).toBeTruthy();

	await page.goto(`http://localhost:5173/accept-invitation?token=${token}`);
	await page.waitForLoadState('networkidle');
	await page.getByRole('button', { name: 'Accept Invitation' }).click();
	await page.getByRole('textbox', { name: 'Last Name' }).fill('User');
	await page.getByRole('textbox', { name: 'Password Show password', exact: true }).fill('Test123!');
	await page.getByRole('textbox', { name: 'Confirm Password Show password' }).fill('Test123!');
	await expect(page.getByRole('button', { name: 'Create Account' })).toBeDisabled();
});

test('accept_invitation_invalid_empty_last_name', async ({ page, browser }) => {
	const token = await sendInvitation(
		browser,
		adminCreds,
		'validation2@logsmart.app',
		'staff',
		BRANCH_NAME
	);
	expect(token).toBeTruthy();

	await page.goto(`http://localhost:5173/accept-invitation?token=${token}`);
	await page.waitForLoadState('networkidle');
	await page.getByRole('button', { name: 'Accept Invitation' }).click();
	await page.getByRole('textbox', { name: 'First Name' }).fill('Test');
	await page.getByRole('textbox', { name: 'Password Show password', exact: true }).fill('Test123!');
	await page.getByRole('textbox', { name: 'Confirm Password Show password' }).fill('Test123!');
	await expect(page.getByRole('button', { name: 'Create Account' })).toBeDisabled();
});

test('accept_invitation_invalid_password_requirements', async ({ page, browser }) => {
	const token = await sendInvitation(
		browser,
		adminCreds,
		'validation3@logsmart.app',
		'staff',
		BRANCH_NAME
	);
	expect(token).toBeTruthy();

	await page.goto(`http://localhost:5173/accept-invitation?token=${token}`);
	await page.waitForLoadState('networkidle');
	await page.getByRole('button', { name: 'Accept Invitation' }).click();
	await page.getByRole('textbox', { name: 'First Name' }).fill('Test');
	await page.getByRole('textbox', { name: 'Last Name' }).fill('User');
	await page.getByRole('textbox', { name: 'Password Show password', exact: true }).fill('weak');
	await page.getByRole('textbox', { name: 'Confirm Password Show password' }).fill('weak');
	await expect(page.getByRole('button', { name: 'Create Account' })).toBeDisabled();
});

test('accept_invitation_invalid_password_mismatch', async ({ page, browser }) => {
	const token = await sendInvitation(
		browser,
		adminCreds,
		'validation4@logsmart.app',
		'staff',
		BRANCH_NAME
	);
	expect(token).toBeTruthy();

	await page.goto(`http://localhost:5173/accept-invitation?token=${token}`);
	await page.waitForLoadState('networkidle');
	await page.getByRole('button', { name: 'Accept Invitation' }).click();
	await page.getByRole('textbox', { name: 'First Name' }).fill('Test');
	await page.getByRole('textbox', { name: 'Last Name' }).fill('User');
	await page.getByRole('textbox', { name: 'Password Show password', exact: true }).fill('Test123!');
	await page.getByRole('textbox', { name: 'Confirm Password Show password' }).fill('Different123!');
	await expect(page.getByRole('button', { name: 'Create Account' })).toBeDisabled();
});

test('accept_invitation_without_token_shows_error', async ({ page }) => {
	await page.goto('http://localhost:5173/accept-invitation');
	await page.waitForLoadState('networkidle');
	await expect(page.locator('body')).toContainText('Invalid invitation link');
});

test('accept_invitation_with_invalid_token_shows_error', async ({ page }) => {
	await page.goto('http://localhost:5173/accept-invitation?token=invalid-token-12345');
	await page.waitForLoadState('networkidle');
	await expect(page.locator('body')).toContainText('Invalid invitation link');
});
