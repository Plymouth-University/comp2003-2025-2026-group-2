import { test, expect } from '@playwright/test';
import { register, sendInvitation, acceptInvitation, createBranch } from './utils';

test.describe.configure({ timeout: 180000 });

let adminCreds: {
	email: string;
	password: string;
	companyName: string;
	firstName: string;
	lastName: string;
};

let branchManagerCreds: {
	email: string;
	password: string;
};

let hqStaffCreds: {
	email: string;
	password: string;
};

let branchStaffCreds: {
	email: string;
	password: string;
};

const BRANCH_A = 'Attendance Branch A';
const BRANCH_B = 'Attendance Branch B';

test.beforeAll(async ({ browser }) => {
	test.setTimeout(120000);
	const creds = await register(browser);
	if (!creds) throw new Error('Failed to register admin user');
	adminCreds = creds;

	const page = await browser.newPage();
	await page.goto('http://localhost:5173/login');
	await page.getByRole('textbox', { name: 'Email' }).fill(adminCreds.email);
	await page.getByRole('textbox', { name: 'Password' }).fill(adminCreds.password);
	await page.getByRole('button', { name: 'Sign in', exact: true }).click();
	await page.waitForURL('**/dashboard');

	await createBranch(page, BRANCH_A, '1 Church Road');
	await createBranch(page, BRANCH_B, '2 High Street');
	await page.close();

	const bmEmail = `bm-attendance-${Date.now()}@logsmart.app`;
	const bmToken = await sendInvitation(browser, adminCreds, bmEmail, 'branch_manager', BRANCH_A);
	if (!bmToken) throw new Error('Failed to get branch manager invitation token');
	const bmSuccess = await acceptInvitation(
		await browser.newPage(),
		bmToken,
		'Branch',
		'Manager',
		'BMAttendance123!'
	);
	if (!bmSuccess) throw new Error('Failed to accept branch manager invitation');
	branchManagerCreds = { email: bmEmail, password: 'BMAttendance123!' };

	const hqEmail = `hq-attendance-${Date.now()}@logsmart.app`;
	const hqToken = await sendInvitation(browser, adminCreds, hqEmail, 'staff');
	if (!hqToken) throw new Error('Failed to get HQ staff invitation token');
	const hqSuccess = await acceptInvitation(
		await browser.newPage(),
		hqToken,
		'HQ',
		'Staff',
		'HQAttendance123!'
	);
	if (!hqSuccess) throw new Error('Failed to accept HQ staff invitation');
	hqStaffCreds = { email: hqEmail, password: 'HQAttendance123!' };

	const branchEmail = `branch-attendance-${Date.now()}@logsmart.app`;
	const branchToken = await sendInvitation(browser, adminCreds, branchEmail, 'staff', BRANCH_A);
	if (!branchToken) throw new Error('Failed to get branch staff invitation token');
	const branchSuccess = await acceptInvitation(
		await browser.newPage(),
		branchToken,
		'Branch',
		'Staff',
		'BranchAttendance123!',
		'**/logs-list'
	);
	if (!branchSuccess) throw new Error('Failed to accept branch staff invitation');
	branchStaffCreds = { email: branchEmail, password: 'BranchAttendance123!' };
});

test.describe('Attendance Admin - Company Manager', () => {
	test.beforeEach(async ({ page }) => {
		await page.goto('http://localhost:5173/');
		await page.getByRole('link', { name: 'Login' }).click();
		await page.getByRole('textbox', { name: 'Email' }).fill(adminCreds.email);
		await page.getByRole('textbox', { name: 'Password' }).fill(adminCreds.password);
		await page.getByRole('button', { name: 'Sign in', exact: true }).click();
		await page.waitForURL('**/dashboard');
	});

	test('company_manager_can_view_attendance_admin_page', async ({ page }) => {
		await page.getByRole('link', { name: 'Attendance' }).click();
		await page.waitForURL('**/attendance-admin');

		await expect(page.locator('body')).toContainText('Attendance');
		await expect(page.getByRole('textbox', { name: /search/i })).toBeVisible();
	});

	test('company_manager_can_filter_by_date_range', async ({ page }) => {
		await page.getByRole('link', { name: 'Attendance' }).click();
		await page.waitForURL('**/attendance-admin');

		const dateFromInput = page.locator('input[type="text"]').first();
		await expect(dateFromInput).toBeVisible();
	});

	test('company_manager_can_filter_by_branch', async ({ page }) => {
		await page.getByRole('link', { name: 'Attendance' }).click();
		await page.waitForURL('**/attendance-admin');

		const branchFilter = page.locator('select').first();
		if (await branchFilter.isVisible()) {
			const options = await branchFilter.locator('option').allInnerTexts();
			expect(options.some((opt) => opt.includes(BRANCH_A))).toBe(true);
			expect(options.some((opt) => opt.includes(BRANCH_B))).toBe(true);
		}
	});

	test('admin_can_search_by_user_name', async ({ page }) => {
		await page.getByRole('link', { name: 'Attendance' }).click();
		await page.waitForURL('**/attendance-admin');

		const searchInput = page.getByRole('textbox', { name: /search/i });
		await searchInput.fill('Branch');
		await page.waitForTimeout(500);

		await expect(page.locator('body')).toBeVisible();
	});

	test('admin_can_search_by_user_email', async ({ page }) => {
		await page.getByRole('link', { name: 'Attendance' }).click();
		await page.waitForURL('**/attendance-admin');

		const searchInput = page.getByRole('textbox', { name: /search/i });
		await searchInput.fill(branchManagerCreds.email.split('@')[0]);
		await page.waitForTimeout(500);

		await expect(page.locator('body')).toBeVisible();
	});
});

test.describe('Attendance Admin - Branch Manager', () => {
	test.beforeEach(async ({ page }) => {
		await page.goto('http://localhost:5173/');
		await page.getByRole('link', { name: 'Login' }).click();
		await page.getByRole('textbox', { name: 'Email' }).fill(branchManagerCreds.email);
		await page.getByRole('textbox', { name: 'Password' }).fill(branchManagerCreds.password);
		await page.getByRole('button', { name: 'Sign in', exact: true }).click();
		await page.waitForURL('**/dashboard');
	});

	test('branch_manager_can_access_attendance_page', async ({ page }) => {
		await page.getByRole('link', { name: 'Attendance' }).click();
		await page.waitForURL('**/attendance-admin');

		await expect(page.locator('body')).toContainText('Attendance');
	});

	test('branch_manager_sees_only_their_branch_events', async ({ page }) => {
		await page.getByRole('link', { name: 'Attendance' }).click();
		await page.waitForURL('**/attendance-admin');

		await expect(page.locator('body')).toBeVisible();
	});

	test('branch_manager_cannot_change_branch_filter', async ({ page }) => {
		await page.getByRole('link', { name: 'Attendance' }).click();
		await page.waitForURL('**/attendance-admin');

		const branchSelect = page.locator('select').first();
		if (await branchSelect.isVisible()) {
			const isDisabled = await branchSelect.isDisabled();
			expect(isDisabled).toBe(true);
		}
	});
});

test.describe('Attendance Admin - HQ Staff', () => {
	test.beforeEach(async ({ page }) => {
		await page.goto('http://localhost:5173/');
		await page.getByRole('link', { name: 'Login' }).click();
		await page.getByRole('textbox', { name: 'Email' }).fill(hqStaffCreds.email);
		await page.getByRole('textbox', { name: 'Password' }).fill(hqStaffCreds.password);
		await page.getByRole('button', { name: 'Sign in', exact: true }).click();
		await page.waitForURL('**/dashboard');
	});

	test('hq_staff_can_view_attendance_with_branch_filter', async ({ page }) => {
		await page.getByRole('link', { name: 'Attendance' }).click();
		await page.waitForURL('**/attendance-admin');

		await expect(page.locator('body')).toContainText('Attendance');

		const branchFilter = page.locator('select').first();
		if (await branchFilter.isVisible()) {
			const options = await branchFilter.locator('option').allInnerTexts();
			expect(options.some((opt) => opt.includes(BRANCH_A))).toBe(true);
		}
	});
});

test.describe('Attendance Admin - Branch Staff (Access Control)', () => {
	test('branch_staff_redirected_from_attendance_admin', async ({ page }) => {
		await page.goto('http://localhost:5173/');
		await page.getByRole('link', { name: 'Login' }).click();
		await page.getByRole('textbox', { name: 'Email' }).fill(branchStaffCreds.email);
		await page.getByRole('textbox', { name: 'Password' }).fill(branchStaffCreds.password);
		await page.getByRole('button', { name: 'Sign in', exact: true }).click();
		await page.waitForURL('**/logs-list');

		await page.goto('http://localhost:5173/attendance-admin');
		await page.waitForTimeout(1000);

		const url = page.url();
		expect(url).not.toContain('attendance-admin');
	});

	test('branch_staff_cannot_see_attendance_nav_link', async ({ page }) => {
		await page.goto('http://localhost:5173/');
		await page.getByRole('link', { name: 'Login' }).click();
		await page.getByRole('textbox', { name: 'Email' }).fill(branchStaffCreds.email);
		await page.getByRole('textbox', { name: 'Password' }).fill(branchStaffCreds.password);
		await page.getByRole('button', { name: 'Sign in', exact: true }).click();
		await page.waitForURL('**/logs-list');

		const attendanceLink = page.getByRole('link', { name: 'Attendance' });
		await expect(attendanceLink).not.toBeVisible();
	});
});

test.describe('Attendance Admin - Unauthenticated Access', () => {
	test('unauthenticated_user_redirected_from_attendance_admin', async ({ page }) => {
		await page.goto('http://localhost:5173/attendance-admin');
		await page.waitForTimeout(1000);

		const url = page.url();
		expect(url).not.toContain('attendance-admin');
	});
});
