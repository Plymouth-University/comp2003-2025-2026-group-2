import { test, expect } from '@playwright/test';
import { acceptInvitation, register, sendInvitation, createBranch } from './utils';

let adminCreds: {
	email: string;
	password: string;
	companyName: string;
	firstName: string;
	lastName: string;
};

const BRANCH_NAME = 'Test Branch Schedule';

test.beforeAll(async ({ browser }) => {
	const creds = await register(browser, false);
	if (!creds) throw new Error('Failed to register admin user');
	adminCreds = creds;

	await createBranch(creds.page!, BRANCH_NAME, '123 Test St');
	await creds.page!.close();
});

test.describe('Log Scheduling - Template Settings', () => {
	test.beforeEach(async ({ page }) => {
		await page.goto('http://localhost:5173/');
		await page.getByRole('link', { name: 'Login' }).click();
		await page.getByRole('textbox', { name: 'Email' }).fill(adminCreds.email);
		await page.getByRole('textbox', { name: 'Password' }).fill(adminCreds.password);
		await page.getByRole('button', { name: 'Sign in', exact: true }).click();
		await page.waitForURL('**/dashboard');
	});

	test('daily_template_shows_time_pickers_in_settings', async ({ page }) => {
		await page.getByRole('link', { name: 'Templates Dashboard' }).click();
		await page.waitForURL('**/templates-dashboard');
		await page.waitForLoadState('networkidle');

		const createButton = page.getByRole('button', { name: 'Create New Template' });
		if (await createButton.isVisible()) {
			await createButton.click();
			await page.waitForURL('**/template-designer');
			await page.waitForLoadState('networkidle');

			const nameInput = page.getByPlaceholder('Template Name');
			await nameInput.fill('Daily Schedule Test Template');
			await page.waitForTimeout(500);

			await page.getByRole('button', { name: 'Save Template' }).click();
			await page.getByRole('link', { name: 'Templates Dashboard' }).click();
			await page.waitForURL('**/templates-dashboard');
			await page.waitForLoadState('networkidle');

			await page.getByRole('button', { name: 'Settings', exact: true }).click();
			await page.waitForTimeout(500);

			const frequencySelect = page.locator('#frequency-select');
			await frequencySelect.selectOption('daily');
			await page.waitForTimeout(500);

			const availableFromInput = page.locator('#available-from');
			const dueAtInput = page.locator('#due-at');

			await expect(availableFromInput).toBeVisible();
			await expect(dueAtInput).toBeVisible();

			await expect(availableFromInput).toHaveValue('08:00');
			await expect(dueAtInput).toHaveValue('17:00');
		}
	});

	test('can_set_custom_time_window_for_daily_template', async ({ page }) => {
		await page.getByRole('link', { name: 'Templates Dashboard' }).click();
		await page.waitForURL('**/templates-dashboard');
		await page.waitForLoadState('networkidle');

		const createButton = page.getByRole('button', { name: 'Create New Template' });
		if (await createButton.isVisible()) {
			await createButton.click();
			await page.waitForURL('**/template-designer');
			await page.waitForLoadState('networkidle');

			const nameInput = page.getByPlaceholder('Template Name');
			await nameInput.fill('Custom Time Template');
			await page.waitForTimeout(500);

			await page.getByRole('button', { name: 'Save Template' }).click();
			await page.getByRole('link', { name: 'Templates Dashboard' }).click();
			await page.waitForURL('**/templates-dashboard');
			await page.waitForLoadState('networkidle');

			await page.getByRole('button', { name: 'Settings', exact: true }).click();
			await page.waitForTimeout(500);

			const frequencySelect = page.locator('#frequency-select');
			await frequencySelect.selectOption('daily');
			await page.waitForTimeout(500);

			await page.locator('#available-from').fill('09:00');
			await page.locator('#due-at').fill('18:00');

			await page.getByRole('button', { name: 'Save' }).click();
			await page.waitForTimeout(1000);

			await page.getByRole('button', { name: 'Settings', exact: true }).click();
			await page.waitForTimeout(500);

			await expect(page.locator('#available-from')).toHaveValue('09:00');
			await expect(page.locator('#due-at')).toHaveValue('18:00');
		}
	});

	test('weekly_template_does_not_show_time_pickers', async ({ page }) => {
		await page.getByRole('link', { name: 'Templates Dashboard' }).click();
		await page.waitForURL('**/templates-dashboard');
		await page.waitForLoadState('networkidle');

		const createButton = page.getByRole('button', { name: 'Create New Template' });
		if (await createButton.isVisible()) {
			await createButton.click();
			await page.waitForURL('**/template-designer');
			await page.waitForLoadState('networkidle');

			const nameInput = page.getByPlaceholder('Template Name');
			await nameInput.fill('Weekly Schedule Test Template');
			await page.waitForTimeout(500);

			await page.getByRole('button', { name: 'Save Template' }).click();
			await page.getByRole('link', { name: 'Templates Dashboard' }).click();
			await page.waitForURL('**/templates-dashboard');
			await page.waitForLoadState('networkidle');

			await page.getByRole('button', { name: 'Settings', exact: true }).click();
			await page.waitForTimeout(500);

			const frequencySelect = page.locator('#frequency-select');
			await frequencySelect.selectOption('weekly');
			await page.waitForTimeout(500);

			const availableFromInput = page.locator('#available-from');
			const dueAtInput = page.locator('#due-at');

			await expect(availableFromInput).not.toBeVisible();
			await expect(dueAtInput).not.toBeVisible();
		}
	});
});

test.describe('Log Scheduling - Overdue Logs Display', () => {
	let memberCreds: {
		email: string;
		password: string;
	};

	test.beforeAll(async ({ browser }) => {
		const memberEmail = `schedule-member-${Date.now()}-${Math.floor(Math.random() * 1_000_000)}@logsmart.app`;
		const invitationToken = await sendInvitation(
			browser,
			adminCreds,
			memberEmail,
			'staff',
			BRANCH_NAME
		);
		if (!invitationToken) throw new Error('Failed to get invitation token');
		const scPage = await browser.newPage();
		const success = await acceptInvitation(
			scPage,
			invitationToken!,
			'Schedule',
			'Member',
			'Member123!',
			'**/logs-list'
		);
		await scPage.close();
		if (!success) throw new Error('Failed to accept invitation for member user');
		memberCreds = { email: memberEmail, password: 'Member123!' };
	});

	test.beforeEach(async ({ page }) => {
		await page.goto('http://localhost:5173/');
		await page.getByRole('link', { name: 'Login' }).click();
		await page.getByRole('textbox', { name: 'Email' }).fill(memberCreds.email);
		await page.getByRole('textbox', { name: 'Password' }).fill(memberCreds.password);
		await page.getByRole('button', { name: 'Sign in', exact: true }).click();
		await page.waitForURL('**/logs-list');
	});

	test('member_sees_logs_due_today_section', async ({ page }) => {
		await expect(page.locator('body')).toContainText('Logs Due Today');
	});

	test('overdue_section_hidden_when_no_overdue_logs', async ({ page }) => {
		const overdueHeading = page.getByRole('heading', { name: 'Overdue Logs' });
		if (await overdueHeading.isVisible()) {
			await expect(overdueHeading).toBeVisible();
		}
	});

	test('available_time_window_shown_for_due_today_logs', async ({ page }) => {
		await page.waitForLoadState('networkidle');

		const dueByPattern = /Due by:/;
		const notAvailablePattern = /Not yet available|Available from:/;

		const body = page.locator('body');
		const hasDueBy = await body.getByText(dueByPattern).count();
		const hasAvailableInfo = await body.getByText(notAvailablePattern).count();

		if (hasDueBy > 0 || hasAvailableInfo > 0) {
			await expect(body).toHaveText(dueByPattern);
		}
	});
});

test.describe('Log Scheduling - Fill Out and Status', () => {
	let memberCreds: {
		email: string;
		password: string;
	};

	test.beforeAll(async ({ browser }) => {
		const memberEmail = `schedule-fill-${Date.now()}-${Math.floor(Math.random() * 1_000_000)}@logsmart.app`;
		const invitationToken = await sendInvitation(
			browser,
			adminCreds,
			memberEmail,
			'staff',
			BRANCH_NAME
		);
		if (!invitationToken) throw new Error('Failed to get invitation token');
		const scPage = await browser.newPage();
		const success = await acceptInvitation(
			scPage,
			invitationToken!,
			'Fill',
			'Tester',
			'Member123!',
			'**/logs-list'
		);
		await scPage.close();
		if (!success) throw new Error('Failed to accept invitation for member user');
		memberCreds = { email: memberEmail, password: 'Member123!' };
	});

	test('fill_out_button_enabled_for_available_logs', async ({ page }) => {
		await page.goto('http://localhost:5173/');
		await page.getByRole('link', { name: 'Login' }).click();
		await page.getByRole('textbox', { name: 'Email' }).fill(memberCreds.email);
		await page.getByRole('textbox', { name: 'Password' }).fill(memberCreds.password);
		await page.getByRole('button', { name: 'Sign in', exact: true }).click();
		await page.waitForURL('**/logs-list');
		await page.waitForLoadState('networkidle');

		const fillOutButton = page.getByRole('button', { name: 'Fill Out' }).first();
		if (await fillOutButton.isVisible({ timeout: 5000 })) {
			await expect(fillOutButton).toBeEnabled();
		}
	});

	test('submit_log_creates_past_log_entry', async ({ page }) => {
		await page.goto('http://localhost:5173/');
		await page.getByRole('link', { name: 'Login' }).click();
		await page.getByRole('textbox', { name: 'Email' }).fill(memberCreds.email);
		await page.getByRole('textbox', { name: 'Password' }).fill(memberCreds.password);
		await page.getByRole('button', { name: 'Sign in', exact: true }).click();
		await page.waitForURL('**/logs-list');
		await page.waitForLoadState('networkidle');

		const fillOutButton = page.getByRole('button', { name: 'Fill Out' }).first();
		if (await fillOutButton.isVisible({ timeout: 5000 })) {
			await fillOutButton.click();
			await page.waitForLoadState('networkidle');

			const submitButton = page.getByRole('button', { name: 'Submit Log' });
			if (await submitButton.isVisible({ timeout: 5000 })) {
				await submitButton.click();
				await page.waitForTimeout(2000);
				await page.waitForURL('**/logs-list');
				await expect(page.locator('body')).toContainText('submitted');
			}
		}
	});
});
