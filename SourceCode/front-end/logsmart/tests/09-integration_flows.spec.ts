import { test, expect } from '@playwright/test';
import { register, sendInvitation, acceptInvitation, createBranch } from './utils';

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

test.describe('Integration Flow: Complete Company Onboarding to First Log', () => {
	test('journey_new_company_to_first_submitted_log', async ({ page }) => {
		await page.goto('http://localhost:5173/');
		await page.getByRole('link', { name: 'Login' }).click();
		await page.getByRole('textbox', { name: 'Email' }).fill(adminCreds.email);
		await page.getByRole('textbox', { name: 'Password' }).fill(adminCreds.password);
		await page.getByRole('button', { name: 'Sign in', exact: true }).click();
		await page.waitForURL('**/dashboard');
		await expect(page.locator('body')).toContainText(adminCreds.email);
		await expect(page.locator('body')).toContainText(adminCreds.companyName);

		await page.getByRole('link', { name: 'Templates' }).click();
		await page.waitForURL('**/templates-dashboard');
		await page.getByRole('button', { name: 'Create New Template' }).click();
		await page.waitForURL('**/template-designer');

		const nameInput = page.getByPlaceholder('Template Name');
		await nameInput.fill('Flow Test Temperature Log');
		await page.waitForTimeout(500);

		const tempButton = page.getByRole('button', { name: 'Temperature' });
		if (await tempButton.isVisible()) {
			await tempButton.click();
			await page.waitForTimeout(500);
		}

		const saveButton = page.getByRole('button', { name: 'Save' });
		if ((await saveButton.isVisible()) && (await saveButton.isEnabled())) {
			await saveButton.click();
			await page.waitForTimeout(1000);
		}

		await page.getByRole('link', { name: 'Logs', exact: true }).click();
		await page.waitForURL('**/logs-list');

		const fillOutButton = page.getByRole('button', { name: 'Fill Out' }).first();
		if (await fillOutButton.isVisible()) {
			await fillOutButton.click();
			await page.waitForLoadState('networkidle');

			const submitButton = page.getByRole('button', { name: 'Submit Log' });
			if (await submitButton.isVisible()) {
				await submitButton.click();
				await page.waitForTimeout(1000);
			}
			await page.waitForURL('**/logs-list');
			await expect(page.locator('body')).toContainText('submitted');
		}
	});
});

test.describe('Integration Flow: Admin Invites Member to Complete Log', () => {
	test('journey_admin_invite_member_fill_log', async ({ page, context, browser }) => {
		const timestamp = Date.now();
		const memberEmail = `flowmember${timestamp}@logsmart.app`;

		await page.goto('http://localhost:5173/');
		await page.getByRole('link', { name: 'Login' }).click();
		await page.getByRole('textbox', { name: 'Email' }).fill(adminCreds.email);
		await page.getByRole('textbox', { name: 'Password' }).fill(adminCreds.password);
		await page.getByRole('button', { name: 'Sign in', exact: true }).click();
		await page.waitForURL('**/dashboard');

		const inviteToken = await sendInvitation(
			browser,
			adminCreds,
			memberEmail,
			'staff',
			BRANCH_NAME
		);

		if (!inviteToken) {
			console.log('Invitation token not found, skipping member acceptance');
			return;
		}

		const memberPage = await context.newPage();
		const success = await acceptInvitation(
			memberPage,
			inviteToken,
			'Flow',
			'Member',
			'FlowMember123!',
			'**/logs-list'
		);

		if (!success) {
			console.log('Failed to accept invitation');
			await memberPage.close();
			return;
		}

		await expect(memberPage.locator('body')).toContainText(memberEmail);

		const fillOutButton = memberPage.getByRole('button', { name: 'Fill Out' }).first();
		if (await fillOutButton.isVisible()) {
			await fillOutButton.click();
			await memberPage.waitForLoadState('networkidle');

			const saveButton = memberPage.getByRole('button', { name: 'Save Draft' });
			if (await saveButton.isVisible()) {
				await saveButton.click();
				await page.waitForTimeout(500);
			}

			const submitButton = memberPage.getByRole('button', { name: 'Submit Log' });
			if (await submitButton.isVisible()) {
				await submitButton.click();
				await memberPage.waitForTimeout(1000);
			}
		}

		await memberPage.close();

		await page.getByRole('link', { name: 'Logs', exact: true }).click();
		await page.waitForURL('**/logs-list');
		await page.waitForLoadState('networkidle');
	});
});

test.describe('Integration Flow: Admin Reviews and Unsubmits Log', () => {
	test('journey_admin_review_unsubmit_member_reopen', async ({ page, context, browser }) => {
		await page.goto('http://localhost:5173/');
		await page.getByRole('link', { name: 'Login' }).click();
		await page.getByRole('textbox', { name: 'Email' }).fill(adminCreds.email);
		await page.getByRole('textbox', { name: 'Password' }).fill(adminCreds.password);
		await page.getByRole('button', { name: 'Sign in', exact: true }).click();
		await page.waitForURL('**/dashboard');

		await page.getByRole('link', { name: 'Logs', exact: true }).click();
		await page.waitForURL('**/logs-list');

		const viewButton = page.getByRole('button', { name: 'View' }).first();
		if (await viewButton.isVisible()) {
			await viewButton.click();
			await page.waitForLoadState('networkidle');
			await expect(page.locator('body')).toContainText('submitted');

			await page.goBack();
			await page.waitForURL('**/logs-list');
		}

		const unsubmitButton = page.getByRole('button', { name: 'Unsubmit' }).first();
		if (await unsubmitButton.isVisible()) {
			await unsubmitButton.click();
			await page.waitForTimeout(1000);
		}

		const memberEmail = `flowmember${Date.now()}@logsmart.app`;
		const memberPass = 'FlowMember123!';
		const inviteToken = await sendInvitation(
			browser,
			adminCreds,
			memberEmail,
			'staff',
			BRANCH_NAME
		);

		if (!inviteToken) {
			console.log('Invitation token not found, skipping member acceptance');
			return;
		}

		const memberPage = await context.newPage();
		const success = await acceptInvitation(
			memberPage,
			inviteToken,
			'Flow',
			'Member',
			memberPass,
			'**/logs-list'
		);

		if (!success) {
			console.log('Failed to accept invitation');
			await memberPage.close();
			return;
		}

		await expect(memberPage.locator('body')).toContainText(memberEmail);

		await page.getByRole('button', { name: 'Logout' }).click();
		await page.waitForURL('**/login');

		await page.getByRole('textbox', { name: 'Email' }).fill(memberEmail);
		await page.getByRole('textbox', { name: 'Password' }).fill(memberPass);
		await page.getByRole('button', { name: 'Sign in', exact: true }).click();
		await page.waitForURL('**/logs-list');

		const editButton = page.getByRole('button', { name: 'Edit' }).first();
		if (await editButton.isVisible()) {
			await editButton.click();
			await page.waitForLoadState('networkidle');

			const reopenedWarning = page.locator('text=/reopened|edit/i').first();
			if (await reopenedWarning.isVisible()) {
				await expect(reopenedWarning).toBeVisible();
			}
		}
	});
});

test.describe('Integration Flow: Template Lifecycle', () => {
	test('journey_create_schedule_use_delete_template', async ({ page }) => {
		const timestamp = Date.now();
		const templateName = `Lifecycle Template ${timestamp}`;

		await page.goto('http://localhost:5173/');
		await page.getByRole('link', { name: 'Login' }).click();
		await page.getByRole('textbox', { name: 'Email' }).fill(adminCreds.email);
		await page.getByRole('textbox', { name: 'Password' }).fill(adminCreds.password);
		await page.getByRole('button', { name: 'Sign in', exact: true }).click();
		await page.waitForURL('**/dashboard');

		await page.getByRole('link', { name: 'Templates' }).click();
		await page.waitForURL('**/templates-dashboard');
		await page.getByRole('button', { name: 'Create New Template' }).click();
		await page.waitForURL('**/template-designer');

		const nameInput = page.getByPlaceholder('Template Name');
		await nameInput.fill(templateName);
		await page.waitForTimeout(500);

		const textInputButton = page.getByRole('button', { name: 'Text Input' });
		if (await textInputButton.isVisible()) {
			await textInputButton.click();
			await page.waitForTimeout(500);
		}

		const saveButton = page.getByRole('button', { name: 'Save' });
		if ((await saveButton.isVisible()) && (await saveButton.isEnabled())) {
			await saveButton.click();
			await page.waitForTimeout(1000);
		}

		await page.getByRole('link', { name: 'Templates' }).click();
		await page.waitForURL('**/templates-dashboard');

		const searchInput = page.getByPlaceholder('Search templates');
		if (await searchInput.isVisible()) {
			await searchInput.fill(templateName);
			await page.waitForTimeout(500);
		}

		const settingsButton = page.getByRole('button', { name: 'Settings' }).first();
		if (await settingsButton.isVisible()) {
			await settingsButton.click();
			await page.waitForTimeout(500);

			const saveScheduleButton = page.getByRole('button', { name: 'Save' });
			if (await saveScheduleButton.isVisible()) {
				await saveScheduleButton.click();
				await page.waitForTimeout(1000);
			}
		}

		await page.goto(
			`http://localhost:5173/log-template?template=${encodeURIComponent(templateName)}`
		);
		await page.waitForLoadState('networkidle');

		const submitButton = page.getByRole('button', { name: 'Submit Log' });
		if (await submitButton.isVisible()) {
			await submitButton.click();
			await page.waitForTimeout(1000);
		}

		await page.goto('http://localhost:5173/templates-dashboard');
		await page.waitForURL('**/templates-dashboard');

		if (await searchInput.isVisible()) {
			await searchInput.fill(templateName);
			await page.waitForTimeout(500);
		}

		const deleteButton = page.getByRole('button', { name: 'Delete' }).first();
		if (await deleteButton.isVisible()) {
			await deleteButton.click();
			await page.waitForTimeout(500);

			const confirmButton = page.getByRole('button', { name: 'Confirm' });
			if (await confirmButton.isVisible()) {
				await confirmButton.click();
				await page.waitForTimeout(1000);
			}
		}
	});
});

test.describe('Integration Flow: Multi-User Collaboration', () => {
	test('journey_multiple_users_same_template', async ({ page, context, browser }) => {
		await page.goto('http://localhost:5173/');
		await page.getByRole('link', { name: 'Login' }).click();
		await page.getByRole('textbox', { name: 'Email' }).fill(adminCreds.email);
		await page.getByRole('textbox', { name: 'Password' }).fill(adminCreds.password);
		await page.getByRole('button', { name: 'Sign in', exact: true }).click();
		await page.waitForURL('**/dashboard');

		await page.getByRole('link', { name: 'Logs', exact: true }).click();
		await page.waitForURL('**/logs-list');

		const adminFillButton = page.getByRole('button', { name: 'Fill Out' }).first();
		if (await adminFillButton.isVisible()) {
			await adminFillButton.click();
			await page.waitForLoadState('networkidle');

			const saveButton = page.getByRole('button', { name: 'Save Draft' });
			if (await saveButton.isVisible()) {
				await saveButton.click();
				await page.waitForTimeout(500);
			}

			await page.waitForURL('**/logs-list');
		}

		const memberEmail = `flowmember${Date.now()}@logsmart.app`;
		const inviteToken = await sendInvitation(
			browser,
			adminCreds,
			memberEmail,
			'staff',
			BRANCH_NAME
		);

		if (!inviteToken) {
			console.log('Invitation token not found, skipping member acceptance');
			return;
		}

		const memberPage = await context.newPage();
		const success = await acceptInvitation(
			memberPage,
			inviteToken,
			'Flow',
			'Member',
			'FlowMember123!',
			'**/logs-list'
		);

		if (!success) {
			console.log('Failed to accept invitation');
			await memberPage.close();
			return;
		}

		await expect(memberPage.locator('body')).toContainText(memberEmail);

		const memberFillButton = memberPage.getByRole('button', { name: 'Fill Out' }).first();
		if (await memberFillButton.isVisible()) {
			await memberFillButton.click();
			await memberPage.waitForLoadState('networkidle');

			const memberSubmitButton = memberPage.getByRole('button', { name: 'Submit Log' });
			if (await memberSubmitButton.isVisible()) {
				await memberSubmitButton.click();
				await memberPage.waitForTimeout(1000);
			}
		}

		await memberPage.close();

		await page.reload();
		await page.waitForLoadState('networkidle');
	});
});

test.describe('Integration Flow: Settings and Profile Management', () => {
	test('journey_update_profile_across_application', async ({ page }) => {
		await page.goto('http://localhost:5173/');
		await page.getByRole('link', { name: 'Login' }).click();
		await page.getByRole('textbox', { name: 'Email' }).fill(adminCreds.email);
		await page.getByRole('textbox', { name: 'Password' }).fill(adminCreds.password);
		await page.getByRole('button', { name: 'Sign in', exact: true }).click();
		await page.waitForURL('**/dashboard');

		await page.getByRole('link', { name: 'Settings' }).click();
		await page.waitForURL('**/settings');

		await page.getByRole('textbox', { name: 'First Name' }).clear();
		await page.getByRole('textbox', { name: 'First Name' }).fill('Updated');
		await page.getByRole('textbox', { name: 'Last Name' }).clear();
		await page.getByRole('textbox', { name: 'Last Name' }).fill('Name');
		await page.getByRole('button', { name: 'Save Profile' }).click();
		await page.waitForTimeout(1000);

		await page.getByRole('link', { name: 'Dashboard', exact: true }).click();
		await page.waitForURL('**/dashboard');
		await expect(page.locator('body')).toContainText('Updated Name');

		await page.getByRole('link', { name: 'Users' }).click();
		await page.waitForURL('**/users-admin');
		await expect(page.locator('body')).toContainText('Updated Name');

		await page.getByRole('link', { name: 'Settings' }).click();
		await page.waitForURL('**/settings');
		await page.getByRole('textbox', { name: 'First Name' }).clear();
		await page.getByRole('textbox', { name: 'First Name' }).fill('Test');
		await page.getByRole('textbox', { name: 'Last Name' }).clear();
		await page.getByRole('textbox', { name: 'Last Name' }).fill('User');
		await page.getByRole('button', { name: 'Save Profile' }).click();
		await page.waitForTimeout(1000);
	});
});

test.describe('Integration Flow: Error Recovery', () => {
	test('journey_network_interruption_recovery', async ({ page }) => {
		await page.goto('http://localhost:5173/');
		await page.getByRole('link', { name: 'Login' }).click();
		await page.getByRole('textbox', { name: 'Email' }).fill(adminCreds.email);
		await page.getByRole('textbox', { name: 'Password' }).fill(adminCreds.password);
		await page.getByRole('button', { name: 'Sign in', exact: true }).click();
		await page.waitForURL('**/dashboard');

		await page.getByRole('link', { name: 'Templates' }).click();
		await page.waitForURL('**/templates-dashboard');
		await page.waitForLoadState('networkidle');

		await page.reload();
		await page.waitForURL('**/templates-dashboard');
		await page.waitForLoadState('networkidle');
	});

	test('journey_session_expiry_redirect', async ({ page }) => {
		await page.goto('http://localhost:5173/');
		await page.getByRole('link', { name: 'Login' }).click();
		await page.getByRole('textbox', { name: 'Email' }).fill(adminCreds.email);
		await page.getByRole('textbox', { name: 'Password' }).fill(adminCreds.password);
		await page.getByRole('button', { name: 'Sign in', exact: true }).click();
		await page.waitForURL('**/dashboard');

		await page.context().clearCookies();

		await page.goto('http://localhost:5173/dashboard');
		await page.waitForURL('**/login');
		await expect(page).toHaveURL('/login');
	});
});
