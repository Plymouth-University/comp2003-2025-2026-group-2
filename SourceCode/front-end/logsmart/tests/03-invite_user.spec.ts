import { test, expect } from '@playwright/test';
import { register, getInvitationToken, sendInvitation } from './utils';

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

	// Verify new fields exist
	await expect(page.locator('#invite-role')).toBeVisible();
	await expect(page.locator('#invite-branch')).toBeVisible();

	await page.getByRole('textbox', { name: "New user's email" }).click();
	const inviteeEmail = `invitee-${Date.now()}@logsmart.app`;
	await page.getByRole('textbox', { name: "New user's email" }).fill(inviteeEmail);
	await page.getByRole('button', { name: 'Send Invite' }).click();

	const invitationToken = await getInvitationToken(inviteeEmail);
	expect(invitationToken).toBeTruthy();
});

test('accept_invitation', async ({ page }) => {
	const inviteeEmail = `invitee-${Date.now()}@logsmart.app`;
	const invitationToken = await sendInvitation(page.context().browser()!, adminCreds, inviteeEmail);
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
