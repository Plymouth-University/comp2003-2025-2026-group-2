import { test, expect } from '@playwright/test';
import { register, createBranch, getInvitationToken, sendInvitation } from './utils';

let adminCreds: {
	email: string;
	password: string;
	companyName: string;
	firstName: string;
	lastName: string;
	page?: any;
};

test.beforeAll(async ({ browser }) => {
	const creds = await register(browser, false);
	if (!creds) throw new Error('Failed to register admin user');
	adminCreds = creds;
	await createBranch(adminCreds.page, 'Main Branch', '123 Main St');
});

test.beforeEach(async ({ browser }) => {
	if (!adminCreds.page) {
		adminCreds.page = await browser.newPage();
		await adminCreds.page.goto('http://localhost:5173/');
		await adminCreds.page.getByRole('link', { name: 'Login' }).click();
		await adminCreds.page.waitForURL('**/login');
		await adminCreds.page.getByRole('textbox', { name: 'Email' }).click();
		await adminCreds.page.getByRole('textbox', { name: 'Email' }).fill(adminCreds.email);
		await adminCreds.page.getByRole('textbox', { name: 'Email' }).press('Tab');
		await adminCreds.page.getByRole('textbox', { name: 'Password' }).fill(adminCreds.password);
		await adminCreds.page.getByRole('button', { name: 'Sign in', exact: true }).click();
		await adminCreds.page.waitForURL('**/dashboard');
	}
});

test('invite_user', async ({ page }) => {
	await page.goto('http://localhost:5173/');
	await page.getByRole('link', { name: 'Login' }).click();
	await page.waitForURL('**/login');
	await page.getByRole('textbox', { name: 'Email' }).click();
	await page.getByRole('textbox', { name: 'Email' }).fill(adminCreds.email);
	await page.getByRole('textbox', { name: 'Email' }).press('Tab');
	await page.getByRole('textbox', { name: 'Password' }).fill(adminCreds.password);
	await page.getByRole('button', { name: 'Sign in', exact: true }).click();
	await page.waitForURL('**/dashboard');

	await page.getByRole('link', { name: 'Users' }).click();
	await expect(page.locator('#eventHide')).toContainText(adminCreds.email);
	await expect(page.locator('#eventHide')).toContainText(adminCreds.firstName.split('-')[0]);
	await expect(page.locator('#eventHide')).toContainText('Company Manager');
	await expect(page.locator('header')).toContainText(adminCreds.email);
	await page.getByRole('button', { name: '➕' }).click();

	await expect(page.locator('#invite-role')).toBeVisible();
	await expect(page.locator('#invite-branch')).toBeVisible();

	await page.getByRole('textbox', { name: "New user's email" }).click();
	const inviteeEmail = `invitee-${Date.now()}@logsmart.app`;
	await page.getByRole('textbox', { name: "New user's email" }).fill(inviteeEmail);
	await page.locator('#invite-branch').selectOption({ label: 'Main Branch' });
	await page.getByRole('button', { name: 'Send Invite' }).click();

	const invitationToken = await getInvitationToken(inviteeEmail);
	expect(invitationToken).toBeTruthy();
});

test('accept_invitation', async ({ page, browser }) => {
	//await createBranch(adminCreds.page, 'Main Branch', '123 Main St');
	const inviteeEmail = `invitee-${Date.now()}@logsmart.app`;
	const invitationToken = await sendInvitation(
		browser,
		adminCreds,
		inviteeEmail,
		'staff',
		'Main Branch'
	);
	expect(invitationToken).toBeTruthy();

	await page.goto(`http://localhost:5173/accept-invitation?token=${invitationToken}`);
	await page.waitForURL('**/accept-invitation**');

	await page.getByRole('button', { name: 'Accept Invitation' }).click();

	await page.getByRole('textbox', { name: 'First Name' }).fill('Test');
	await page.getByRole('textbox', { name: 'Last Name' }).fill('Member');

	await page
		.getByRole('textbox', { name: 'Password Show password', exact: true })
		.fill('Member123!');
	await page.getByRole('textbox', { name: 'Confirm Password' }).fill('Member123!');
	await page.getByRole('button', { name: 'Create Account' }).click();

	await page.waitForURL('**/logs-list');
	expect(page.url()).toContain('/logs-list');
	await expect(page.locator('span')).toContainText(inviteeEmail);
});
