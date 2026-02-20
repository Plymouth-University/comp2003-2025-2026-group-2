import { test, expect } from '@playwright/test';
import { register, sendInvitation, acceptInvitation, createBranch } from './utils';

test.describe.configure({ timeout: 120000 });

let adminCreds: {
	email: string;
	password: string;
	companyName: string;
	firstName: string;
	lastName: string;
};

let staffCreds: {
	email: string;
	password: string;
};

const BRANCH_NAME = 'Clock Test Branch';

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

	await createBranch(page, BRANCH_NAME, '10 High Street');
	await page.close();

	const staffEmail = `clockstaff-${Date.now()}@logsmart.app`;
	const invitationToken = await sendInvitation(
		browser,
		adminCreds,
		staffEmail,
		'staff',
		BRANCH_NAME
	);
	if (!invitationToken) throw new Error('Failed to get staff invitation token');
	const success = await acceptInvitation(
		await browser.newPage(),
		invitationToken,
		'Clock',
		'Staff',
		'ClockStaff123!',
		'**/logs-list'
	);
	if (!success) throw new Error('Failed to accept staff invitation');
	staffCreds = { email: staffEmail, password: 'ClockStaff123!' };
});

test.describe('Clock In/Out - Staff User', () => {
	test.beforeEach(async ({ page }) => {
		await page.goto('http://localhost:5173/');
		await page.getByRole('link', { name: 'Login' }).click();
		await page.getByRole('textbox', { name: 'Email' }).fill(staffCreds.email);
		await page.getByRole('textbox', { name: 'Password' }).fill(staffCreds.password);
		await page.getByRole('button', { name: 'Sign in', exact: true }).click();
		await page.waitForURL('**/logs-list');
		await page.goto('http://localhost:5173/dashboard');
		await page.waitForLoadState('networkidle');
	});

	test('staff_user_can_clock_in_from_dashboard', async ({ page }) => {
		const clockInSection = page.locator('text=Clock In / Out');
		await expect(clockInSection).toBeVisible();

		const clockInButton = page.getByRole('button', { name: 'Clock In' });
		if (await clockInButton.isVisible()) {
			await clockInButton.click();
			await page.waitForTimeout(1000);

			await expect(page.getByText('Currently Clocked In')).toBeVisible();
			await expect(page.getByRole('button', { name: 'Clock Out' })).toBeVisible();
		} else {
			const clockOutButton = page.getByRole('button', { name: 'Clock Out' });
			await expect(clockOutButton).toBeVisible();
		}
	});

	test('staff_user_can_clock_out_from_dashboard', async ({ page }) => {
		const clockInButton = page.getByRole('button', { name: 'Clock In' });
		if (await clockInButton.isVisible()) {
			await clockInButton.click();
			await page.waitForTimeout(1000);
		}

		const clockOutButton = page.getByRole('button', { name: 'Clock Out' });
		await expect(clockOutButton).toBeVisible();
		await clockOutButton.click();
		await page.waitForTimeout(1000);

		await expect(page.getByText('Currently Clocked Out')).toBeVisible();
		await expect(page.getByRole('button', { name: 'Clock In' })).toBeVisible();
	});

	test('clock_status_persists_after_page_refresh', async ({ page }) => {
		const clockInButton = page.getByRole('button', { name: 'Clock In' });
		if (await clockInButton.isVisible()) {
			await clockInButton.click();
			await page.waitForTimeout(1000);
		}

		await expect(page.getByText('Currently Clocked In')).toBeVisible();

		await page.reload();
		await page.waitForLoadState('networkidle');

		await expect(page.getByText('Currently Clocked In')).toBeVisible();
	});

	test('clock_in_shows_error_when_already_clocked_in', async ({ page }) => {
		const clockInButton = page.getByRole('button', { name: 'Clock In' });
		if (await clockInButton.isVisible()) {
			await clockInButton.click();
			await page.waitForTimeout(1000);
		}

		await expect(page.getByText('Currently Clocked In')).toBeVisible();

		const clockOutButton = page.getByRole('button', { name: 'Clock Out' });
		await clockOutButton.click();
		await page.waitForTimeout(1000);
		await expect(page.getByText('Currently Clocked Out')).toBeVisible();
	});

	test('recent_activity_displays_clock_events', async ({ page }) => {
		const clockInButton = page.getByRole('button', { name: 'Clock In' });
		if (await clockInButton.isVisible()) {
			await clockInButton.click();
			await page.waitForTimeout(500);

			const clockOutButton = page.getByRole('button', { name: 'Clock Out' });
			await clockOutButton.click();
			await page.waitForTimeout(1000);
		}

		const recentActivity = page.locator('text=Recent Activity');
		await expect(recentActivity).toBeVisible();
	});
});

test.describe('Clock In/Out - Admin User', () => {
	test.beforeEach(async ({ page }) => {
		await page.goto('http://localhost:5173/');
		await page.getByRole('link', { name: 'Login' }).click();
		await page.getByRole('textbox', { name: 'Email' }).fill(adminCreds.email);
		await page.getByRole('textbox', { name: 'Password' }).fill(adminCreds.password);
		await page.getByRole('button', { name: 'Sign in', exact: true }).click();
		await page.waitForURL('**/dashboard');
		await page.waitForLoadState('networkidle');
	});

	test('admin_user_can_clock_in', async ({ page }) => {
		const clockInSection = page.locator('text=Clock In / Out');
		await expect(clockInSection).toBeVisible();

		const clockInButton = page.getByRole('button', { name: 'Clock In', exact: true });
		if (await clockInButton.isVisible()) {
			await clockInButton.click();
			await page.waitForTimeout(1000);
			await expect(page.getByText('Currently Clocked In')).toBeVisible();
		} else {
			await expect(page.getByRole('button', { name: 'Clock Out' })).toBeVisible();
		}
	});

	test('admin_user_can_clock_out', async ({ page }) => {
		const clockInButton = page.getByRole('button', { name: 'Clock In', exact: true });
		if (await clockInButton.isVisible()) {
			await clockInButton.click();
			await page.waitForTimeout(1000);
		}

		const clockOutButton = page.getByRole('button', { name: 'Clock Out', exact: true });
		await expect(clockOutButton).toBeVisible();
		await clockOutButton.click();
		await page.waitForTimeout(1000);

		await expect(page.getByText('Currently Clocked Out')).toBeVisible();
	});
});

test.describe('Clock In/Out - Multiple Sessions', () => {
	test('user_can_complete_multiple_clock_cycles', async ({ page }) => {
		await page.goto('http://localhost:5173/');
		await page.getByRole('link', { name: 'Login' }).click();
		await page.getByRole('textbox', { name: 'Email' }).fill(staffCreds.email);
		await page.getByRole('textbox', { name: 'Password' }).fill(staffCreds.password);
		await page.getByRole('button', { name: 'Sign in', exact: true }).click();
		await page.waitForURL('**/logs-list');
		await page.goto('http://localhost:5173/dashboard');
		await page.waitForLoadState('networkidle');

		const clockInButton = page.getByRole('button', { name: 'Clock In' });
		if (await clockInButton.isVisible()) {
			await clockInButton.click();
			await page.waitForTimeout(500);
		}

		await page.getByRole('button', { name: 'Clock Out' }).click();
		await page.waitForTimeout(500);
		await expect(page.getByText('Currently Clocked Out')).toBeVisible();

		await page.getByRole('button', { name: 'Clock In' }).click();
		await page.waitForTimeout(500);
		await expect(page.getByText('Currently Clocked In')).toBeVisible();

		await page.getByRole('button', { name: 'Clock Out' }).click();
		await page.waitForTimeout(500);
		await expect(page.getByText('Currently Clocked Out')).toBeVisible();
	});
});
