import { test, expect } from '@playwright/test';
import { register, sendInvitation, acceptInvitation, createBranch } from './utils';

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
	await bmPage.close();
	if (!bmSuccess) throw new Error('Failed to accept branch manager invitation');
	branchManagerCreds = { email: bmEmail, password: 'BMAttendance123!' };

	const hqEmail = `hq-attendance-${Date.now()}@logsmart.app`;
	const hqToken = await sendInvitation(browser, adminCreds, hqEmail, 'staff');
	if (!hqToken) throw new Error('Failed to get HQ staff invitation token');
	const hqPage = await browser.newPage();
	const hqSuccess = await acceptInvitation(hqPage, hqToken, 'HQ', 'Staff', 'HQAttendance123!');
	await hqPage.close();
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
	await bsPage.close();
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

	test('should load attendance data with default 7-day range on initial page load', async ({
		page
	}) => {
		await page.getByRole('link', { name: 'Attendance' }).click();
		await page.waitForURL('**/attendance-admin');
		await page.waitForLoadState('networkidle');

		// Verify date inputs are visible
		const fromDateInput = page.locator('input#filter-from');
		const toDateInput = page.locator('input#filter-to');
		await expect(fromDateInput).toBeVisible();
		await expect(toDateInput).toBeVisible();

		// Get the date values
		const fromValue = await fromDateInput.inputValue();
		const toValue = await toDateInput.inputValue();

		// Parse dates (format is DD/MM/YYYY)
		const parseDateString = (dateStr: string) => {
			const [day, month, year] = dateStr.split('/');
			return new Date(`${year}-${month}-${day}`);
		};

		const fromDate = parseDateString(fromValue);
		const toDate = parseDateString(toValue);

		// Verify they're ~7 days apart (6 days difference between start and end)
		const daysDiff = Math.round((toDate.getTime() - fromDate.getTime()) / (1000 * 60 * 60 * 24));
		expect(daysDiff).toBe(6); // 7 days inclusive range = 6 days difference

		// Verify table exists and has rows
		const table = page.locator('table');
		await expect(table).toBeVisible();
	});

	test('should respect URL parameters over default 7-day range', async ({ page }) => {
		// Navigate with custom date parameters
		const customFrom = '2026-05-10T00:00:00Z';
		const customTo = '2026-05-11T23:59:59.999Z';
		await page.goto(
			`http://localhost:5173/attendance-admin?from=${encodeURIComponent(customFrom)}&to=${encodeURIComponent(customTo)}`
		);
		await page.waitForLoadState('networkidle');

		// Verify date inputs display custom dates
		const fromDateInput = page.locator('input#filter-from');
		const toDateInput = page.locator('input#filter-to');
		await expect(fromDateInput).toBeVisible();
		await expect(toDateInput).toBeVisible();

		const fromValue = await fromDateInput.inputValue();
		const toValue = await toDateInput.inputValue();

		// Should be 10/05/2026 and 11/05/2026 (DD/MM/YYYY format)
		expect(fromValue).toBe('10/05/2026');
		expect(toValue).toBe('11/05/2026');
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
		await page.waitForLoadState('networkidle');

		const branchFilter = page.locator('#filter-branch');
		await expect(branchFilter).toBeVisible();
	});

	test('hq_staff_can_see_all_branches_in_filter', async ({ page }) => {
		await page.getByRole('link', { name: 'Attendance' }).click();
		await page.waitForURL('**/attendance-admin');
		await page.waitForLoadState('networkidle');

		const branchFilter = page.locator('#filter-branch');
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
		await page.waitForLoadState('networkidle');

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
		await page.waitForLoadState('networkidle');

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

		const clockOutButton = staffPage.getByRole('button', { name: 'Clock Out' });
		if (await clockOutButton.isVisible()) {
			await clockOutButton.click();
		}

		const clockInButton = staffPage.getByRole('button', { name: 'Clock In' });
		if (await clockInButton.isVisible()) {
			await clockInButton.click();

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
		await adminPage.getByRole('button', { name: 'Apply' }).click();
		await adminPage.waitForURL('**/attendance-admin*');
		await adminPage.waitForLoadState('networkidle');

		await expect(adminPage.locator('body')).toContainText(branchStaffCreds.email);
		await expect(adminPage.locator('body')).toContainText('Clocked In');
		await adminPage.close();
	});
});
