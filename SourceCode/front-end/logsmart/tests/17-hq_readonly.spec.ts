import { test, expect } from '@playwright/test';
import {
	register,
	createBranch,
	sendInvitation,
	acceptInvitation,
	clearMailhogEmails
} from './utils';

let adminCreds: {
	email: string;
	password: string;
	companyName: string;
	firstName: string;
	lastName: string;
};

const BRANCH_NAME = 'Test Branch';

test.beforeAll(async ({ browser }) => {
	await clearMailhogEmails();
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

test.describe('HQ Readonly Staff - User Management', () => {
	let hqStaffCreds: {
		email: string;
		password: string;
	};

	test.beforeAll(async ({ browser }) => {
		const hqEmail = `hqstaff-${Date.now()}@logsmart.app`;
		const invitationToken = await sendInvitation(browser, adminCreds, hqEmail, 'staff');
		if (!invitationToken) throw new Error('Failed to get HQ staff invitation token');
		const success = await acceptInvitation(
			await browser.newPage(),
			invitationToken,
			'HQ',
			'Staff',
			'HQStaff123!'
		);
		if (!success) throw new Error('Failed to accept HQ staff invitation');
		hqStaffCreds = { email: hqEmail, password: 'HQStaff123!' };
	});

	test('hq_staff_can_view_users_page', async ({ page }) => {
		await page.goto('http://localhost:5173/');
		await page.getByRole('link', { name: 'Login' }).click();
		await page.getByRole('textbox', { name: 'Email' }).fill(hqStaffCreds.email);
		await page.getByRole('textbox', { name: 'Password' }).fill(hqStaffCreds.password);
		await page.getByRole('button', { name: 'Sign in', exact: true }).click();
		await page.waitForURL('**/dashboard');

		await page.getByRole('link', { name: 'Users' }).click();
		await page.waitForURL('**/users-admin');

		await expect(page.locator('body')).toContainText(adminCreds.email);
		await expect(page.locator('body')).toContainText('Company Manager');
	});

	test('hq_staff_cannot_see_add_user_button', async ({ page }) => {
		await page.goto('http://localhost:5173/');
		await page.getByRole('link', { name: 'Login' }).click();
		await page.getByRole('textbox', { name: 'Email' }).fill(hqStaffCreds.email);
		await page.getByRole('textbox', { name: 'Password' }).fill(hqStaffCreds.password);
		await page.getByRole('button', { name: 'Sign in', exact: true }).click();
		await page.waitForURL('**/dashboard');

		await page.getByRole('link', { name: 'Users' }).click();
		await page.waitForURL('**/users-admin');

		await expect(page.getByRole('button', { name: '➕' })).not.toBeVisible();
	});

	test('hq_staff_cannot_see_edit_delete_buttons', async ({ page }) => {
		await page.goto('http://localhost:5173/');
		await page.getByRole('link', { name: 'Login' }).click();
		await page.getByRole('textbox', { name: 'Email' }).fill(hqStaffCreds.email);
		await page.getByRole('textbox', { name: 'Password' }).fill(hqStaffCreds.password);
		await page.getByRole('button', { name: 'Sign in', exact: true }).click();
		await page.waitForURL('**/dashboard');

		await page.getByRole('link', { name: 'Users' }).click();
		await page.waitForURL('**/users-admin');

		await expect(page.getByRole('button', { name: 'Edit' })).not.toBeVisible();
		await expect(page.getByRole('button', { name: 'Delete' })).not.toBeVisible();
	});
});

test.describe('HQ Readonly Staff - Branch Management', () => {
	let hqStaffCreds: {
		email: string;
		password: string;
	};

	test.beforeAll(async ({ browser }) => {
		const hqEmail = `hqbranch-${Date.now()}@logsmart.app`;
		const invitationToken = await sendInvitation(browser, adminCreds, hqEmail, 'staff');
		if (!invitationToken) throw new Error('Failed to get HQ staff invitation token');
		const success = await acceptInvitation(
			await browser.newPage(),
			invitationToken,
			'HQ',
			'Branch',
			'HQBranch123!'
		);
		if (!success) throw new Error('Failed to accept HQ staff invitation');
		hqStaffCreds = { email: hqEmail, password: 'HQBranch123!' };
	});

	test('hq_staff_can_view_branches_page', async ({ page }) => {
		await page.goto('http://localhost:5173/');
		await page.getByRole('link', { name: 'Login' }).click();
		await page.getByRole('textbox', { name: 'Email' }).fill(hqStaffCreds.email);
		await page.getByRole('textbox', { name: 'Password' }).fill(hqStaffCreds.password);
		await page.getByRole('button', { name: 'Sign in', exact: true }).click();
		await page.waitForURL('**/dashboard');

		await page.getByRole('link', { name: 'Branches' }).click();
		await page.waitForURL('**/branches');

		await expect(page.locator('body')).toContainText(BRANCH_NAME);
	});

	test('hq_staff_cannot_see_add_branch_button', async ({ page }) => {
		await page.goto('http://localhost:5173/');
		await page.getByRole('link', { name: 'Login' }).click();
		await page.getByRole('textbox', { name: 'Email' }).fill(hqStaffCreds.email);
		await page.getByRole('textbox', { name: 'Password' }).fill(hqStaffCreds.password);
		await page.getByRole('button', { name: 'Sign in', exact: true }).click();
		await page.waitForURL('**/dashboard');

		await page.getByRole('link', { name: 'Branches' }).click();
		await page.waitForURL('**/branches');

		await expect(page.getByRole('button', { name: 'ADD BRANCH' })).not.toBeVisible();
	});

	test('hq_staff_cannot_see_edit_delete_buttons_on_branch', async ({ page }) => {
		await page.goto('http://localhost:5173/');
		await page.getByRole('link', { name: 'Login' }).click();
		await page.getByRole('textbox', { name: 'Email' }).fill(hqStaffCreds.email);
		await page.getByRole('textbox', { name: 'Password' }).fill(hqStaffCreds.password);
		await page.getByRole('button', { name: 'Sign in', exact: true }).click();
		await page.waitForURL('**/dashboard');

		await page.getByRole('link', { name: 'Branches' }).click();
		await page.waitForURL('**/branches');

		await expect(page.getByRole('button', { name: /Edit|✏️/i })).not.toBeVisible();
		await expect(page.getByRole('button', { name: /Delete|🗑️/i })).not.toBeVisible();
	});
});

test.describe('HQ Readonly Staff - Template Dashboard', () => {
	let hqStaffCreds: {
		email: string;
		password: string;
	};

	test.beforeAll(async ({ browser }) => {
		const hqEmail = `hqtemplates-${Date.now()}@logsmart.app`;
		const invitationToken = await sendInvitation(browser, adminCreds, hqEmail, 'staff');
		if (!invitationToken) throw new Error('Failed to get HQ staff invitation token');
		const success = await acceptInvitation(
			await browser.newPage(),
			invitationToken,
			'HQ',
			'Templates',
			'HQTemplates123!'
		);
		if (!success) throw new Error('Failed to accept HQ staff invitation');
		hqStaffCreds = { email: hqEmail, password: 'HQTemplates123!' };
	});

	test('hq_staff_can_view_templates_dashboard', async ({ page }) => {
		await page.goto('http://localhost:5173/');
		await page.getByRole('link', { name: 'Login' }).click();
		await page.getByRole('textbox', { name: 'Email' }).fill(hqStaffCreds.email);
		await page.getByRole('textbox', { name: 'Password' }).fill(hqStaffCreds.password);
		await page.getByRole('button', { name: 'Sign in', exact: true }).click();
		await page.waitForURL('**/dashboard');

		await page.getByRole('link', { name: 'Templates Dashboard' }).click();
		await page.waitForURL('**/templates-dashboard');
	});

	test('hq_staff_cannot_see_create_template_button', async ({ page }) => {
		await page.goto('http://localhost:5173/');
		await page.getByRole('link', { name: 'Login' }).click();
		await page.getByRole('textbox', { name: 'Email' }).fill(hqStaffCreds.email);
		await page.getByRole('textbox', { name: 'Password' }).fill(hqStaffCreds.password);
		await page.getByRole('button', { name: 'Sign in', exact: true }).click();
		await page.waitForURL('**/dashboard');

		await page.getByRole('link', { name: 'Templates Dashboard' }).click();
		await page.waitForURL('**/templates-dashboard');

		await expect(page.getByRole('button', { name: 'Create New Template' })).not.toBeVisible();
	});
});

test.describe('HQ Readonly Staff - Logs', () => {
	let hqStaffCreds: {
		email: string;
		password: string;
	};

	test.beforeAll(async ({ browser }) => {
		const hqEmail = `hqlogs-${Date.now()}@logsmart.app`;
		const invitationToken = await sendInvitation(browser, adminCreds, hqEmail, 'staff');
		if (!invitationToken) throw new Error('Failed to get HQ staff invitation token');
		const success = await acceptInvitation(
			await browser.newPage(),
			invitationToken,
			'HQ',
			'Logs',
			'HQLogs123!'
		);
		if (!success) throw new Error('Failed to accept HQ staff invitation');
		hqStaffCreds = { email: hqEmail, password: 'HQLogs123!' };
	});

	test('hq_staff_can_access_reports_page', async ({ page }) => {
		await page.goto('http://localhost:5173/');
		await page.getByRole('link', { name: 'Login' }).click();
		await page.getByRole('textbox', { name: 'Email' }).fill(hqStaffCreds.email);
		await page.getByRole('textbox', { name: 'Password' }).fill(hqStaffCreds.password);
		await page.getByRole('button', { name: 'Sign in', exact: true }).click();
		await page.waitForURL('**/dashboard');

		await page.getByRole('link', { name: 'Reports' }).click();
		await page.waitForURL('**/reports');
	});
});

test.describe('HQ Staff - Invite Without Branch Creates HQ', () => {
	test('inviting_staff_without_branch_creates_hq_account', async ({ browser }) => {
		const hqEmail = `hqauto-${Date.now()}@logsmart.app`;
		const invitationToken = await sendInvitation(browser, adminCreds, hqEmail, 'staff');
		expect(invitationToken).toBeTruthy();

		const page = await browser.newPage();
		await page.goto(`http://localhost:5173/accept-invitation?token=${invitationToken}`);
		await page.waitForURL('**/accept-invitation**');

		await page.getByRole('button', { name: 'Accept Invitation' }).click();
		await page.getByRole('textbox', { name: 'First Name' }).fill('Auto');
		await page.getByRole('textbox', { name: 'Last Name' }).fill('HQ');
		await page
			.getByRole('textbox', { name: 'Password Show password', exact: true })
			.fill('AutoHQ123!');
		await page.getByRole('textbox', { name: 'Confirm Password' }).fill('AutoHQ123!');
		await page.getByRole('button', { name: 'Create Account' }).click();

		await page.waitForURL('**/dashboard', { timeout: 10000 });

		await page.getByRole('link', { name: 'Users' }).click();
		await page.waitForURL('**/users-admin');
		await expect(page.locator('body')).toContainText(hqEmail);
		await expect(page.locator('body')).toContainText('Staff');

		await page.getByRole('link', { name: 'Branches' }).click();
		await page.waitForURL('**/branches');
		await expect(page.getByRole('button', { name: 'ADD BRANCH' })).not.toBeVisible();

		await page.close();
	});
});
