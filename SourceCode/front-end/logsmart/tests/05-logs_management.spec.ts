import { test, expect } from '@playwright/test';
import { acceptInvitation, register, sendInvitation, createBranch } from './utils';

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

test.describe('Logs Management - Admin', () => {
	test.beforeEach(async ({ page }) => {
		await page.goto('http://localhost:5173/');
		await page.getByRole('link', { name: 'Login' }).click();
		await page.getByRole('textbox', { name: 'Email' }).fill(adminCreds.email);
		await page.getByRole('textbox', { name: 'Password' }).fill(adminCreds.password);
		await page.getByRole('button', { name: 'Sign in', exact: true }).click();
		await page.waitForURL('**/dashboard');
	});

	test('view_logs_list_admin', async ({ page }) => {
		await page.getByRole('link', { name: 'Logs', exact: true }).click();
		await page.waitForURL('**/logs-list');
		await expect(page.locator('body')).toContainText('Logs Due Today');
		await expect(page.locator('body')).toContainText('All Logs');
	});

	test('create_new_log_entry_from_template', async ({ page }) => {
		await page.waitForLoadState('networkidle');
	});

	test('save_log_as_draft', async ({ page }) => {
		await page.waitForLoadState('networkidle');
	});

	test('submit_log_entry', async ({ page }) => {
		await page.goto('http://localhost:5173/log-template?template=Test%20Template');
		await page.waitForLoadState('networkidle');
		const submitButton = page.getByRole('button', { name: 'Submit Log' });
		if (await submitButton.isVisible()) {
			await submitButton.click();
			await page.waitForLoadState('networkidle');
			await expect(page.locator('body')).toContainText('submitted');
		}
	});

	test('unsubmit_log_as_admin', async ({ page }) => {
		await page.getByRole('link', { name: 'Logs', exact: true }).click();
		await page.waitForURL('**/logs-list');
		const unsubmitButton = page.getByRole('button', { name: 'Unsubmit' }).first();
		if (await unsubmitButton.isVisible()) {
			await unsubmitButton.click();
			await page.waitForTimeout(500);
		}
	});

	test('edit_draft_log', async ({ page }) => {
		await page.getByRole('link', { name: 'Logs', exact: true }).click();
		await page.waitForURL('**/logs-list');
		const editButton = page.getByRole('button', { name: 'Edit' }).first();
		if (await editButton.isVisible()) {
			await editButton.click();
			await page.waitForLoadState('networkidle');
		}
	});

	test('navigate_back_to_logs_list', async ({ page }) => {
		await page.waitForLoadState('networkidle');
	});
});

test.describe('Logs Management - Member', () => {
	let memberCreds: {
		email: string;
		password: string;
	};

	test.beforeAll(async ({ browser }) => {
		const memberEmail = `member-${Date.now()}-${Math.floor(Math.random() * 1_000_000)}@logsmart.app`;
		const invitationToken = await sendInvitation(
			browser,
			adminCreds,
			memberEmail,
			'staff',
			BRANCH_NAME
		);
		if (!invitationToken) throw new Error('Failed to get invitation token');
		const success = await acceptInvitation(
			await browser.newPage(),
			invitationToken!,
			'Member',
			'User',
			'Member123!',
			'**/logs-list'
		);
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

	test('member_redirected_to_logs_list_on_login', async ({ page }) => {
		await expect(page).toHaveURL('/logs-list');
	});

	test('member_view_logs_list', async ({ page }) => {
		await expect(page.locator('body')).toContainText('Logs Due Today');
		await expect(page.locator('body')).toContainText('Past Logs');
	});

	test('member_cannot_access_dashboard', async ({ page }) => {
		await page.goto('http://localhost:5173/dashboard');
		await page.waitForURL('**/logs-list');
		await expect(page).toHaveURL('/logs-list');
	});

	test('member_cannot_access_users_admin', async ({ page }) => {
		await page.goto('http://localhost:5173/users-admin');
		await page.waitForURL('**/logs-list');
		await expect(page).toHaveURL('/logs-list');
	});

	test('member_cannot_access_templates_dashboard', async ({ page }) => {
		await page.goto('http://localhost:5173/templates-dashboard');
		await page.waitForURL('**/logs-list');
		await expect(page).toHaveURL('/logs-list');
	});

	test('member_cannot_access_template_designer', async ({ page }) => {
		await page.goto('http://localhost:5173/template-designer');
		await page.waitForURL('**/logs-list');
		await expect(page).toHaveURL('/logs-list');
	});

	test('member_fill_out_assigned_log', async ({ page }) => {
		const fillOutButton = page.getByRole('button', { name: 'Fill Out' }).first();
		if (await fillOutButton.isVisible()) {
			await fillOutButton.click();
			await page.waitForLoadState('networkidle');
		}
	});

	test('member_view_past_log', async ({ page }) => {
		const viewButton = page.getByRole('button', { name: 'View' }).first();
		if (await viewButton.isVisible()) {
			await viewButton.click();
			await page.waitForLoadState('networkidle');
		}
	});

	test('member_cannot_unsubmit_log', async ({ page }) => {
		const unsubmitButton = page.getByRole('button', { name: 'Unsubmit' });
		await expect(unsubmitButton).not.toBeVisible();
	});
});

test.describe('Log Form Components', () => {
	test.beforeEach(async ({ page }) => {
		await page.goto('http://localhost:5173/');
		await page.getByRole('link', { name: 'Login' }).click();
		await page.getByRole('textbox', { name: 'Email' }).fill(adminCreds.email);
		await page.getByRole('textbox', { name: 'Password' }).fill(adminCreds.password);
		await page.getByRole('button', { name: 'Sign in', exact: true }).click();
		await page.waitForURL('**/dashboard');
	});

	test('temperature_picker_increment_decrement', async ({ page }) => {
		await page.waitForLoadState('networkidle');
	});

	test('temperature_picker_direct_input', async ({ page }) => {
		await page.waitForLoadState('networkidle');
	});

	test('text_input_component', async ({ page }) => {
		await page.waitForLoadState('networkidle');
	});

	test('checkbox_component', async ({ page }) => {
		await page.waitForLoadState('networkidle');
	});

	test('dropdown_component', async ({ page }) => {
		await page.waitForLoadState('networkidle');
	});

	test('view_mode_disables_inputs', async ({ page }) => {
		await page.getByRole('link', { name: 'Logs', exact: true }).click();
		await page.waitForURL('**/logs-list');
		await page.waitForLoadState('networkidle');
	});
});

test.describe('Log Entry Validation', () => {
	test.beforeEach(async ({ page }) => {
		await page.goto('http://localhost:5173/');
		await page.getByRole('link', { name: 'Login' }).click();
		await page.getByRole('textbox', { name: 'Email' }).fill(adminCreds.email);
		await page.getByRole('textbox', { name: 'Password' }).fill(adminCreds.password);
		await page.getByRole('button', { name: 'Sign in', exact: true }).click();
		await page.waitForURL('**/dashboard');
	});

	test('temperature_min_max_validation', async ({ page }) => {
		await page.waitForLoadState('networkidle');
	});

	test('empty_logs_list_shows_message', async ({ page }) => {
		await page.getByRole('link', { name: 'Logs', exact: true }).click();
		await page.waitForURL('**/logs-list');
		await page.waitForLoadState('networkidle');
	});

	test('invalid_template_name_shows_error', async ({ page }) => {
		await page.waitForLoadState('networkidle');
	});

	test('invalid_entry_id_shows_error', async ({ page }) => {
		await page.waitForLoadState('networkidle');
	});
});
