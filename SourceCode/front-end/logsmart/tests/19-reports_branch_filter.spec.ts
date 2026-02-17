import { test, expect } from '@playwright/test';
import { register, createBranch, sendInvitation, acceptInvitation } from './utils';

let adminCreds: {
	email: string;
	password: string;
	companyName: string;
	firstName: string;
	lastName: string;
};

const BRANCH_A = 'Branch Alpha';
const BRANCH_B = 'Branch Beta';

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

	await createBranch(page, BRANCH_A, '100 Alpha St');
	await createBranch(page, BRANCH_B, '200 Beta St');
	await page.close();
});

test.describe('Company Manager - Reports Branch Filter', () => {
	test('company_manager_can_see_branch_filter_on_reports', async ({ page }) => {
		await page.goto('http://localhost:5173/');
		await page.getByRole('link', { name: 'Login' }).click();
		await page.getByRole('textbox', { name: 'Email' }).fill(adminCreds.email);
		await page.getByRole('textbox', { name: 'Password' }).fill(adminCreds.password);
		await page.getByRole('button', { name: 'Sign in', exact: true }).click();
		await page.waitForURL('**/dashboard');

		await page.getByRole('link', { name: 'Reports' }).click();
		await page.waitForURL('**/reports');

		await expect(page.getByText('Filter by Branch')).toBeVisible();
	});

	test('company_manager_can_filter_by_single_branch', async ({ page }) => {
		await page.goto('http://localhost:5173/');
		await page.getByRole('link', { name: 'Login' }).click();
		await page.getByRole('textbox', { name: 'Email' }).fill(adminCreds.email);
		await page.getByRole('textbox', { name: 'Password' }).fill(adminCreds.password);
		await page.getByRole('button', { name: 'Sign in', exact: true }).click();
		await page.waitForURL('**/dashboard');

		await page.getByRole('link', { name: 'Reports' }).click();
		await page.waitForURL('**/reports');

		await expect(page.getByText('Filter by Branch')).toBeVisible();

		const branchFilterButton = page.getByRole('button', { name: /All Branches|Branch/i });
		await branchFilterButton.click();

		await expect(page.getByText(BRANCH_A)).toBeVisible();
		await expect(page.getByText(BRANCH_B)).toBeVisible();
	});
});

test.describe('HQ Staff - Reports Branch Filter', () => {
	let hqStaffCreds: {
		email: string;
		password: string;
	};

	test.beforeAll(async ({ browser }) => {
		const hqEmail = `hqreports-${Date.now()}@logsmart.app`;
		const invitationToken = await sendInvitation(browser, adminCreds, hqEmail, 'staff');
		if (!invitationToken) throw new Error('Failed to get HQ staff invitation token');
		const success = await acceptInvitation(
			await browser.newPage(),
			invitationToken,
			'HQ',
			'Reports',
			'HQReports123!'
		);
		if (!success) throw new Error('Failed to accept HQ staff invitation');
		hqStaffCreds = { email: hqEmail, password: 'HQReports123!' };
	});

	test('hq_staff_can_see_branch_filter_on_reports', async ({ page }) => {
		await page.goto('http://localhost:5173/');
		await page.getByRole('link', { name: 'Login' }).click();
		await page.getByRole('textbox', { name: 'Email' }).fill(hqStaffCreds.email);
		await page.getByRole('textbox', { name: 'Password' }).fill(hqStaffCreds.password);
		await page.getByRole('button', { name: 'Sign in', exact: true }).click();
		await page.waitForURL('**/dashboard');

		await page.getByRole('link', { name: 'Reports' }).click();
		await page.waitForURL('**/reports');

		await expect(page.getByText('Filter by Branch')).toBeVisible();
	});

	test('hq_staff_can_filter_reports_by_branch', async ({ page }) => {
		await page.goto('http://localhost:5173/');
		await page.getByRole('link', { name: 'Login' }).click();
		await page.getByRole('textbox', { name: 'Email' }).fill(hqStaffCreds.email);
		await page.getByRole('textbox', { name: 'Password' }).fill(hqStaffCreds.password);
		await page.getByRole('button', { name: 'Sign in', exact: true }).click();
		await page.waitForURL('**/dashboard');

		await page.getByRole('link', { name: 'Reports' }).click();
		await page.waitForURL('**/reports');

		await expect(page.getByText('Filter by Branch')).toBeVisible();

		const branchFilterButton = page.getByRole('button', { name: /All Branches|Branch/i });
		await branchFilterButton.click();

		await expect(page.getByText(BRANCH_A)).toBeVisible();
		await expect(page.getByText(BRANCH_B)).toBeVisible();
	});
});

test.describe('Branch Staff - No Reports Filter', () => {
	let branchStaffCreds: {
		email: string;
		password: string;
	};

	test.beforeAll(async ({ browser }) => {
		const staffEmail = `staffreports-${Date.now()}@logsmart.app`;
		const invitationToken = await sendInvitation(
			browser,
			adminCreds,
			staffEmail,
			'staff',
			BRANCH_A
		);
		if (!invitationToken) throw new Error('Failed to get staff invitation token');
		const success = await acceptInvitation(
			await browser.newPage(),
			invitationToken,
			'Staff',
			'Reports',
			'StaffReports123!'
		);
		if (!success) throw new Error('Failed to accept staff invitation');
		branchStaffCreds = { email: staffEmail, password: 'StaffReports123!' };
	});

	test('branch_staff_cannot_see_branch_filter_on_reports', async ({ page }) => {
		await page.goto('http://localhost:5173/');
		await page.getByRole('link', { name: 'Login' }).click();
		await page.getByRole('textbox', { name: 'Email' }).fill(branchStaffCreds.email);
		await page.getByRole('textbox', { name: 'Password' }).fill(branchStaffCreds.password);
		await page.getByRole('button', { name: 'Sign in', exact: true }).click();
		await page.waitForURL('**/logs-list');

		await page.getByRole('link', { name: 'Reports' }).click();
		await page.waitForURL('**/reports');

		await expect(page.getByText('Filter by Branch')).not.toBeVisible();
	});

	test('branch_staff_redirected_to_logs_not_reports', async ({ page }) => {
		await page.goto('http://localhost:5173/');
		await page.getByRole('link', { name: 'Login' }).click();
		await page.getByRole('textbox', { name: 'Email' }).fill(branchStaffCreds.email);
		await page.getByRole('textbox', { name: 'Password' }).fill(branchStaffCreds.password);
		await page.getByRole('button', { name: 'Sign in', exact: true }).click();

		await page.waitForURL('**/logs-list');

		await expect(page.getByRole('link', { name: 'Reports' })).not.toBeVisible();
	});
});

test.describe('Branch Manager - No Reports Filter', () => {
	let branchManagerCreds: {
		email: string;
		password: string;
	};

	test.beforeAll(async ({ browser }) => {
		const bmEmail = `bmreports-${Date.now()}@logsmart.app`;
		const invitationToken = await sendInvitation(
			browser,
			adminCreds,
			bmEmail,
			'branch_manager',
			BRANCH_B
		);
		if (!invitationToken) throw new Error('Failed to get BM invitation token');
		const success = await acceptInvitation(
			await browser.newPage(),
			invitationToken,
			'Branch',
			'Manager',
			'BMReports123!'
		);
		if (!success) throw new Error('Failed to accept BM invitation');
		branchManagerCreds = { email: bmEmail, password: 'BMReports123!' };
	});

	test('branch_manager_cannot_see_branch_filter_on_reports', async ({ page }) => {
		await page.goto('http://localhost:5173/');
		await page.getByRole('link', { name: 'Login' }).click();
		await page.getByRole('textbox', { name: 'Email' }).fill(branchManagerCreds.email);
		await page.getByRole('textbox', { name: 'Password' }).fill(branchManagerCreds.password);
		await page.getByRole('button', { name: 'Sign in', exact: true }).click();
		await page.waitForURL('**/dashboard');

		await expect(page.getByRole('link', { name: 'Reports' })).not.toBeVisible();
	});
});
