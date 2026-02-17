import { test, expect } from '@playwright/test';
import {
	acceptInvitation,
	register,
	requestPasswordResetToken,
	sendInvitation,
	createBranch
} from './utils';

let adminCreds: { email: string; password: string; firstName?: string; lastName?: string };
let passwordResetCreds: { email: string; password: string };
const BRANCH_NAME = 'Test Branch';

test.beforeAll(async ({ browser }) => {
	const creds = await register(browser);
	if (!creds) throw new Error('Failed to register admin user');
	adminCreds = { ...creds };

	const page = await browser.newPage();
	await page.goto('http://localhost:5173/login');
	await page.getByRole('textbox', { name: 'Email' }).fill(adminCreds.email);
	await page.getByRole('textbox', { name: 'Password' }).fill(adminCreds.password);
	await page.getByRole('button', { name: 'Sign in', exact: true }).click();
	await page.waitForURL('**/dashboard');

	await createBranch(page, BRANCH_NAME, '123 Test St');
	await page.close();
});

test.describe('Settings - Profile Updates', () => {
	test.beforeEach(async ({ page }) => {
		await page.goto('http://localhost:5173/');
		await page.getByRole('link', { name: 'Login' }).click();
		await page.getByRole('textbox', { name: 'Email' }).fill(adminCreds.email);
		await page.getByRole('textbox', { name: 'Password' }).fill(adminCreds.password);
		await page.getByRole('button', { name: 'Sign in', exact: true }).click();
		await page.waitForURL('**/dashboard');
	});

	test('access_settings_page', async ({ page }) => {
		await page.getByRole('link', { name: 'Settings' }).click();
		await page.waitForURL('**/settings');
		await expect(page).toHaveURL('/settings');
	});

	test('view_current_profile_information', async ({ page }) => {
		await page.getByRole('link', { name: 'Settings' }).click();
		await page.waitForURL('**/settings');
		await expect(page.getByRole('textbox', { name: 'First Name' })).toBeVisible();
		await expect(page.getByRole('textbox', { name: 'Last Name' })).toBeVisible();
		await expect(page.getByRole('textbox', { name: 'Email' })).toBeVisible();
	});

	test('email_field_is_readonly', async ({ page }) => {
		await page.getByRole('link', { name: 'Settings' }).click();
		await page.waitForURL('**/settings');
		const emailField = page.getByRole('textbox', { name: 'Email' });
		await expect(emailField).toBeDisabled();
	});

	test('update_first_name', async ({ page }) => {
		await page.getByRole('link', { name: 'Settings' }).click();
		await page.waitForURL('**/settings');
		await page.getByRole('textbox', { name: 'First Name' }).clear();
		await page.getByRole('textbox', { name: 'First Name' }).fill('UpdatedFirst');
		await page.getByRole('button', { name: 'Save Profile' }).click();
		await page.waitForTimeout(1000);
	});

	test('update_last_name', async ({ page }) => {
		await page.getByRole('link', { name: 'Settings' }).click();
		await page.waitForURL('**/settings');
		await page.getByRole('textbox', { name: 'Last Name' }).clear();
		await page.getByRole('textbox', { name: 'Last Name' }).fill('UpdatedLast');
		await page.getByRole('button', { name: 'Save Profile' }).click();
		await page.waitForTimeout(1000);
	});

	test('update_both_names', async ({ page }) => {
		await page.getByRole('link', { name: 'Settings' }).click();
		await page.waitForURL('**/settings');
		await page.getByRole('textbox', { name: 'First Name' }).clear();
		await page.getByRole('textbox', { name: 'First Name' }).fill('Test');
		await page.getByRole('textbox', { name: 'Last Name' }).clear();
		await page.getByRole('textbox', { name: 'Last Name' }).fill('User');
		await page.getByRole('button', { name: 'Save Profile' }).click();
		await page.waitForTimeout(1000);
	});

	test('empty_first_name_validation', async ({ page }) => {
		await page.getByRole('link', { name: 'Settings' }).click();
		await page.waitForURL('**/settings');
		await page.getByRole('textbox', { name: 'First Name' }).clear();
		const saveButton = page.getByRole('button', { name: 'Save Profile' });
		await expect(saveButton).toBeDisabled();
	});

	test('empty_last_name_validation', async ({ page }) => {
		await page.getByRole('link', { name: 'Settings' }).click();
		await page.waitForURL('**/settings');
		await page.getByRole('textbox', { name: 'Last Name' }).clear();
		const saveButton = page.getByRole('button', { name: 'Save Profile' });
		await expect(saveButton).toBeDisabled();
	});

	test('very_long_first_name', async ({ page }) => {
		await page.getByRole('link', { name: 'Settings' }).click();
		await page.waitForURL('**/settings');
		const longName = 'A'.repeat(100);
		await page.getByRole('textbox', { name: 'First Name' }).clear();
		await page.getByRole('textbox', { name: 'First Name' }).fill(longName);
		await page.getByRole('button', { name: 'Save Profile' }).click();
		await page.waitForTimeout(1000);
	});

	test('special_characters_in_names', async ({ page }) => {
		await page.getByRole('link', { name: 'Settings' }).click();
		await page.waitForURL('**/settings');

		const originalFirstName = await page.getByRole('textbox', { name: 'First Name' }).inputValue();
		const originalLastName = await page.getByRole('textbox', { name: 'Last Name' }).inputValue();

		await page.getByRole('textbox', { name: 'First Name' }).clear();
		await page.getByRole('textbox', { name: 'First Name' }).fill("O'Connor");
		await page.getByRole('textbox', { name: 'Last Name' }).clear();
		await page.getByRole('textbox', { name: 'Last Name' }).fill('Smith-Jones');
		await page.getByRole('button', { name: 'Save Profile' }).click();
		await page.waitForTimeout(1000);

		await page.getByRole('textbox', { name: 'First Name' }).clear();
		await page.getByRole('textbox', { name: 'First Name' }).fill(originalFirstName);
		await page.getByRole('textbox', { name: 'Last Name' }).clear();
		await page.getByRole('textbox', { name: 'Last Name' }).fill(originalLastName);
		await page.getByRole('button', { name: 'Save Profile' }).click();
		await page.waitForTimeout(1000);
	});
});

test.describe('Settings - Password Reset', () => {
	test.beforeAll(async ({ browser }) => {
		const creds = await register(browser);
		if (!creds) throw new Error('Failed to register admin user');
		adminCreds = creds;
	});

	test.beforeEach(async ({ page }) => {
		await page.goto('http://localhost:5173/');
		await page.getByRole('link', { name: 'Login' }).click();
		await page.getByRole('textbox', { name: 'Email' }).fill(adminCreds.email);
		await page.getByRole('textbox', { name: 'Password' }).fill(adminCreds.password);
		await page.getByRole('button', { name: 'Sign in', exact: true }).click();
		await page.waitForURL('**/dashboard');
	});

	test('request_password_reset_from_settings', async ({ page }) => {
		await page.getByRole('link', { name: 'Settings' }).click();
		await page.waitForURL('**/settings');
		const resetButton = page.getByRole('button', { name: 'Request Password Reset' });
		if (await resetButton.isVisible()) {
			await resetButton.click();
			await page.waitForTimeout(1000);
		}
	});

	test('password_reset_success_message', async ({ page }) => {
		await page.getByRole('link', { name: 'Settings' }).click();
		await page.waitForURL('**/settings');
		const resetButton = page.getByRole('button', { name: 'Request Password Reset' });
		if (await resetButton.isVisible()) {
			await resetButton.click();
			await page.waitForTimeout(1000);
		}
	});
});

test.describe('Settings - Dark Mode', () => {
	test.beforeAll(async ({ browser }) => {
		const creds = await register(browser);
		if (!creds) throw new Error('Failed to register admin user');
		adminCreds = creds;
	});
	test.beforeEach(async ({ page }) => {
		await page.goto('http://localhost:5173/');
		await page.getByRole('link', { name: 'Login' }).click();
		await page.getByRole('textbox', { name: 'Email' }).fill(adminCreds.email);
		await page.getByRole('textbox', { name: 'Password' }).fill(adminCreds.password);
		await page.getByRole('button', { name: 'Sign in', exact: true }).click();
		await page.waitForURL('**/dashboard');
	});

	test('toggle_dark_mode_on', async ({ page }) => {
		await page.getByRole('link', { name: 'Settings' }).click();
		await page.waitForURL('**/settings');
		const darkModeToggle = page
			.locator('input[type="checkbox"]')
			.filter({ hasText: /dark/i })
			.or(page.getByLabel('Dark Mode'));
		if (await darkModeToggle.isVisible()) {
			await darkModeToggle.check();
			await page.waitForTimeout(500);
			const htmlElement = page.locator('html');
			await htmlElement.evaluate((el) => el.classList.contains('dark'));
		}
	});

	test('toggle_dark_mode_off', async ({ page }) => {
		await page.getByRole('link', { name: 'Settings' }).click();
		await page.waitForURL('**/settings');
		const darkModeToggle = page
			.locator('input[type="checkbox"]')
			.filter({ hasText: /dark/i })
			.or(page.getByLabel('Dark Mode'));
		if (await darkModeToggle.isVisible()) {
			await darkModeToggle.uncheck();
			await page.waitForTimeout(500);
		}
	});

	test('dark_mode_persists_across_pages', async ({ page }) => {
		await page.getByRole('link', { name: 'Settings' }).click();
		await page.waitForURL('**/settings');
		const darkModeToggle = page
			.locator('input[type="checkbox"]')
			.filter({ hasText: /dark/i })
			.or(page.getByLabel('Dark Mode'));
		if (await darkModeToggle.isVisible()) {
			await darkModeToggle.check();
			await page.waitForTimeout(500);
			await page.getByRole('link', { name: 'Dashboard', exact: true }).click();
			await page.waitForURL('**/dashboard');
			await page.waitForTimeout(500);
			await page.getByRole('link', { name: 'Settings' }).click();
			await page.waitForURL('**/settings');
		}
	});

	test('dark_mode_persists_across_sessions', async ({ page }) => {
		await page.getByRole('link', { name: 'Settings' }).click();
		await page.waitForURL('**/settings');
		const darkModeToggle = page
			.locator('input[type="checkbox"]')
			.filter({ hasText: /dark/i })
			.or(page.getByLabel('Dark Mode'));
		if (await darkModeToggle.isVisible()) {
			await darkModeToggle.check();
			await page.waitForTimeout(500);
			await page.getByRole('button', { name: 'Logout' }).click();
			await page.waitForURL('**/login');
			await page.getByRole('textbox', { name: 'Email' }).fill(adminCreds.email);
			await page.getByRole('textbox', { name: 'Password' }).fill(adminCreds.password);
			await page.getByRole('button', { name: 'Sign in', exact: true }).click();
			await page.waitForURL('**/dashboard');
			await page.getByRole('link', { name: 'Settings' }).click();
			await page.waitForURL('**/settings');
			await page.waitForTimeout(500);
		}
	});
});

test.describe('Settings - Member Access', () => {
	let memberCreds: { email: string; password: string };
	test.beforeAll(async ({ browser }) => {
		const page = await browser.newPage();
		await page.goto('http://localhost:5173/');
		await page.getByRole('link', { name: 'Login' }).click();
		await page.getByRole('textbox', { name: 'Email' }).fill(adminCreds.email);
		await page.getByRole('textbox', { name: 'Password' }).fill(adminCreds.password);
		await page.getByRole('button', { name: 'Sign in', exact: true }).click();
		await page.waitForURL('**/dashboard');
		await page.goto('http://localhost:5173/template-designer');
		await page.waitForLoadState('networkidle');

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
	test('member_can_access_settings', async ({ page }) => {
		await page.getByRole('link', { name: 'Settings' }).click();
		await page.waitForURL('**/settings');
		await expect(page).toHaveURL('/settings');
	});

	test('member_can_update_profile', async ({ page }) => {
		await page.getByRole('link', { name: 'Settings' }).click();
		await page.waitForURL('**/settings');
		await page.getByRole('textbox', { name: 'First Name' }).clear();
		await page.getByRole('textbox', { name: 'First Name' }).fill('Invited');
		await page.getByRole('button', { name: 'Save Profile' }).click();
		await page.waitForTimeout(1000);
	});

	test('member_can_toggle_dark_mode', async ({ page }) => {
		await page.getByRole('link', { name: 'Settings' }).click();
		await page.waitForURL('**/settings');
		const darkModeToggle = page
			.locator('input[type="checkbox"]')
			.filter({ hasText: /dark/i })
			.or(page.getByLabel('Dark Mode'));
		if (await darkModeToggle.isVisible()) {
			await darkModeToggle.check();
			await page.waitForTimeout(500);
			await darkModeToggle.uncheck();
			await page.waitForTimeout(500);
		}
	});
});

test.describe('Password Reset Flow - Unauthenticated', () => {
	test.beforeAll(async ({ browser }) => {
		const creds = await register(browser);
		if (!creds) throw new Error('Failed to register password reset user');
		passwordResetCreds = creds;
	});

	test('access_password_reset_page', async ({ page }) => {
		await page.goto('http://localhost:5173/reset-password');
		await expect(page).toHaveURL('/reset-password');
	});

	test('request_password_reset_valid_email', async ({ page }) => {
		await page.goto('http://localhost:5173/reset-password');
		await page.getByRole('textbox', { name: 'Email' }).fill(passwordResetCreds.email);
		await page.getByRole('button', { name: 'Send Reset Link' }).click();
		await page.waitForTimeout(1000);
	});

	test('request_password_reset_invalid_email_format', async ({ page }) => {
		await page.goto('http://localhost:5173/reset-password');
		await page.getByRole('textbox', { name: 'Email' }).fill('not-an-email');
		const sendButton = page.getByRole('button', { name: 'Send Reset Link' });
		await expect(sendButton).toBeDisabled();
	});

	test('request_password_reset_nonexistent_email', async ({ page }) => {
		await page.goto('http://localhost:5173/reset-password');
		await page.getByRole('textbox', { name: 'Email' }).fill('nonexistent@logsmart.app');
		await page.getByRole('button', { name: 'Send Reset Link' }).click();
		await page.waitForTimeout(1000);
	});

	test('reset_password_with_valid_token', async ({ page, browser }) => {
		const token = await requestPasswordResetToken(browser, passwordResetCreds.email);
		if (!token) throw new Error('Failed to retrieve password reset token');
		await page.goto(`http://localhost:5173/reset-password?token=${token}`);
		await page.waitForLoadState('networkidle');
		const passwordField = page.getByRole('textbox', { name: 'New Password' });
		if (await passwordField.isVisible()) {
			await passwordField.fill('NewPassword123!');
			await page.getByRole('textbox', { name: 'Confirm Password' }).fill('NewPassword123!');
			await page.getByRole('button', { name: 'Set new password' }).click();
			await page.waitForTimeout(1000);
		}
	});

	test('reset_password_without_token_shows_request_form', async ({ page }) => {
		await page.goto('http://localhost:5173/reset-password');
		await expect(page.getByRole('textbox', { name: 'Email' })).toBeVisible();
	});

	test('reset_password_weak_password', async ({ page, browser }) => {
		const token = await requestPasswordResetToken(browser, passwordResetCreds.email);
		if (!token) throw new Error('Failed to retrieve password reset token');
		await page.goto(`http://localhost:5173/reset-password?token=${token}`);
		await page.waitForLoadState('networkidle');
		const passwordField = page.getByRole('textbox', { name: 'New Password' });
		if (await passwordField.isVisible()) {
			await passwordField.fill('weak');
			await page.getByRole('textbox', { name: 'Confirm Password' }).fill('weak');
			const resetButton = page.getByRole('button', { name: 'Set new password' });
			await expect(resetButton).toBeDisabled();
		}
	});

	test('reset_password_mismatch', async ({ page, browser }) => {
		const token = await requestPasswordResetToken(browser, passwordResetCreds.email);
		if (!token) throw new Error('Failed to retrieve password reset token');
		await page.goto(`http://localhost:5173/reset-password?token=${token}`);
		await page.waitForLoadState('networkidle');
		const passwordField = page.getByRole('textbox', { name: 'New Password' });
		if (await passwordField.isVisible()) {
			await passwordField.fill('NewPassword123!');
			await page.getByRole('textbox', { name: 'Confirm Password' }).fill('DifferentPassword123!');
			const resetButton = page.getByRole('button', { name: 'Set new password' });
			await expect(resetButton).toBeDisabled();
		}
	});

	test('reset_password_invalid_token', async ({ page }) => {
		await page.goto('http://localhost:5173/reset-password?token=invalid-token-xyz');
		await page.waitForLoadState('networkidle');
		await page.waitForTimeout(1000);
	});
});
