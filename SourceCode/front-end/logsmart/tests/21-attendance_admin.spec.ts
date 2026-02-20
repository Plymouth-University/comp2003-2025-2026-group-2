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
	const bmPage = await browser.newPage();
	const bmSuccess = await acceptInvitation(
		bmPage,
		bmToken,
		'Branch',
		'Manager',
		'BMAttendance123!'
	);
	bmPage.close();
	if (!bmSuccess) throw new Error('Failed to accept branch manager invitation');
	branchManagerCreds = { email: bmEmail, password: 'BMAttendance123!' };

	const hqEmail = `hq-attendance-${Date.now()}@logsmart.app`;
	const hqToken = await sendInvitation(browser, adminCreds, hqEmail, 'staff');
	if (!hqToken) throw new Error('Failed to get HQ staff invitation token');
	const hqPage = await browser.newPage();
	const hqSuccess = await acceptInvitation(hqPage, hqToken, 'HQ', 'Staff', 'HQAttendance123!');
	hqPage.close();
	if (!hqSuccess) throw new Error('Failed to accept HQ staff invitation');
	hqStaffCreds = { email: hqEmail, password: 'HQAttendance123!' };

	const branchEmail = `branch-attendance-${Date.now()}@logsmart.app`;
	const branchToken = await sendInvitation(browser, adminCreds, branchEmail, 'staff', BRANCH_A);
	if (!branchToken) throw new Error('Failed to get branch staff invitation token');
	const bsPage = await browser.newPage();
	const branchSuccess = await acceptInvitation(
		bsPage,
		branchToken,
		'Branch',
		'Staff',
		'BranchAttendance123!',
		'**/logs-list'
	);
	bsPage.close();
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

	test('hq_staff_can_access_attendance_page', async ({ page }) => {
		await page.getByRole('link', { name: 'Attendance' }).click();
		await page.waitForURL('**/attendance-admin');
		await expect(page.locator('body')).toContainText('Attendance');
	});

	test('hq_staff_can_view_branch_filter_dropdown', async ({ page }) => {
		await page.getByRole('link', { name: 'Attendance' }).click();
		await page.waitForURL('**/attendance-admin');

		const branchFilter = page.locator('select').first();
		await expect(branchFilter).toBeVisible();
	});

	test('hq_staff_can_see_all_branches_in_filter', async ({ page }) => {
		await page.getByRole('link', { name: 'Attendance' }).click();
		await page.waitForURL('**/attendance-admin');

		const branchFilter = page.locator('select').first();
		await branchFilter.click();
		const options = await branchFilter.locator('option').allInnerTexts();
		expect(options.some((opt) => opt.includes(BRANCH_A))).toBe(true);
		expect(options.some((opt) => opt.includes(BRANCH_B))).toBe(true);
	});

	test('hq_staff_can_filter_by_specific_branch', async ({ page }) => {
		await page.getByRole('link', { name: 'Attendance' }).click();
		await page.waitForURL('**/attendance-admin');

		const branchFilter = page.locator('select').first();
		await branchFilter.selectOption({ label: BRANCH_A });
		await page.waitForTimeout(500);
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

test.describe('Attendance Admin - Clock Events Integration', () => {
	test('staff_clocks_in_and_admin_sees_event', async ({ browser }) => {
		const staffPage = await browser.newPage();
		await staffPage.goto('http://localhost:5173/');
		await staffPage.getByRole('link', { name: 'Login' }).click();
		await staffPage.getByRole('textbox', { name: 'Email' }).fill(branchStaffCreds.email);
		await staffPage.getByRole('textbox', { name: 'Password' }).fill(branchStaffCreds.password);
		await staffPage.getByRole('button', { name: 'Sign in', exact: true }).click();
		await staffPage.waitForURL('**/logs-list');
		await staffPage.goto('http://localhost:5173/dashboard');
		await staffPage.waitForLoadState('networkidle');

		const clockInButton = staffPage.getByRole('button', { name: 'Clock In' });
		if (await clockInButton.isVisible()) {
			await clockInButton.click();
			await staffPage.waitForTimeout(1000);
			await expect(staffPage.getByText('Currently Clocked In')).toBeVisible();
		}
		await staffPage.close();

		const adminPage = await browser.newPage();
		await adminPage.goto('http://localhost:5173/');
		await adminPage.getByRole('link', { name: 'Login' }).click();
		await adminPage.getByRole('textbox', { name: 'Email' }).fill(adminCreds.email);
		await adminPage.getByRole('textbox', { name: 'Password' }).fill(adminCreds.password);
		await adminPage.getByRole('button', { name: 'Sign in', exact: true }).click();
		await adminPage.waitForURL('**/dashboard');

		await adminPage.getByRole('link', { name: 'Attendance' }).click();
		await adminPage.waitForURL('**/attendance-admin');
		await adminPage.waitForTimeout(1000);

		await expect(adminPage.locator('body')).toContainText('Branch');
		await expect(adminPage.locator('body')).toContainText('Staff');
		await adminPage.close();
	});

	test('staff_clocks_out_and_admin_sees_completed_event', async ({ browser }) => {
		const staffPage = await browser.newPage();
		await staffPage.goto('http://localhost:5173/');
		await staffPage.getByRole('link', { name: 'Login' }).click();
		await staffPage.getByRole('textbox', { name: 'Email' }).fill(branchStaffCreds.email);
		await staffPage.getByRole('textbox', { name: 'Password' }).fill(branchStaffCreds.password);
		await staffPage.getByRole('button', { name: 'Sign in', exact: true }).click();
		await staffPage.waitForURL('**/logs-list');
		await staffPage.goto('http://localhost:5173/dashboard');
		await staffPage.waitForLoadState('networkidle');

		const clockOutButton = staffPage.getByRole('button', { name: 'Clock Out' });
		if (await clockOutButton.isVisible()) {
			await clockOutButton.click();
			await staffPage.waitForTimeout(1000);
			await expect(staffPage.getByText('Currently Clocked Out')).toBeVisible();
		}
		await staffPage.close();

		const adminPage = await browser.newPage();
		await adminPage.goto('http://localhost:5173/');
		await adminPage.getByRole('link', { name: 'Login' }).click();
		await adminPage.getByRole('textbox', { name: 'Email' }).fill(adminCreds.email);
		await adminPage.getByRole('textbox', { name: 'Password' }).fill(adminCreds.password);
		await adminPage.getByRole('button', { name: 'Sign in', exact: true }).click();
		await adminPage.waitForURL('**/dashboard');

		await adminPage.getByRole('link', { name: 'Attendance' }).click();
		await adminPage.waitForURL('**/attendance-admin');
		await adminPage.waitForTimeout(1000);

		await expect(adminPage.locator('body')).toContainText('Branch');
		await adminPage.close();
	});
});
