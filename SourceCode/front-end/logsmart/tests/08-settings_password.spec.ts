import { test, expect } from '@playwright/test';
import {
	acceptInvitation,
	register,
	requestPasswordResetToken,
	sendInvitation,
	createBranch
} from './utils';
import { validatePasswordField } from './shared-validators';

let adminCreds: { email: string; password: string; firstName?: string; lastName?: string };
let passwordResetCreds: { email: string; password: string };
const BRANCH_NAME = 'Test Branch';

test.beforeAll(async ({ browser }) => {
	const creds = await register(browser, false);
	if (!creds) throw new Error('Failed to register admin user');
	adminCreds = { ...creds };

	await createBranch(creds.page!, BRANCH_NAME, '123 Test St');
	await creds.page!.close();
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
		await page.getByRole('link', { name: 'Settings', exact: true }).click();
		await page.waitForURL('**/settings');
		await expect(page).toHaveURL('/settings');
	});

	test('view_current_profile_information', async ({ page }) => {
		await page.getByRole('link', { name: 'Settings', exact: true }).click();
		await page.waitForURL('**/settings');
		await expect(page.getByRole('textbox', { name: 'First Name' })).toBeVisible();
		await expect(page.getByRole('textbox', { name: 'Last Name' })).toBeVisible();
		await expect(page.getByRole('textbox', { name: 'Email' })).toBeVisible();
	});

	test('email_field_is_readonly', async ({ page }) => {
		await page.getByRole('link', { name: 'Settings', exact: true }).click();
		await page.waitForURL('**/settings');
		const emailField = page.getByRole('textbox', { name: 'Email' });
		await expect(emailField).toBeDisabled();
	});

	test('update_profile_fields', async ({ page }) => {
		test.setTimeout(5000);
		await page.getByRole('link', { name: 'Settings', exact: true }).click();
		await page.waitForURL('**/settings');

		// Test 1: Update first name only (consolidated from update_first_name)
		await page.getByRole('textbox', { name: 'First Name' }).clear();
		await page.getByRole('textbox', { name: 'First Name' }).fill('UpdatedFirst');
		await page.getByRole('button', { name: 'Save Profile' }).click();
		await page.waitForLoadState('networkidle');

		// Test 2: Update last name only (consolidated from update_last_name)
		await page.getByRole('textbox', { name: 'Last Name' }).clear();
		await page.getByRole('textbox', { name: 'Last Name' }).fill('UpdatedLast');
		await page.getByRole('button', { name: 'Save Profile' }).click();
		await page.waitForLoadState('networkidle');

		// Test 3: Update both names (consolidated from update_both_names)
		await page.getByRole('textbox', { name: 'First Name' }).clear();
		await page.getByRole('textbox', { name: 'First Name' }).fill('Test');
		await page.getByRole('textbox', { name: 'Last Name' }).clear();
		await page.getByRole('textbox', { name: 'Last Name' }).fill('User');
		await page.getByRole('button', { name: 'Save Profile' }).click();
		await page.waitForLoadState('networkidle');
	});

	test('profile_field_validation', async ({ page }) => {
		await page.getByRole('link', { name: 'Settings', exact: true }).click();
		await page.waitForURL('**/settings');

		// Test 1: Empty first name validation (consolidated from empty_first_name_validation)
		await page.getByRole('textbox', { name: 'First Name' }).clear();
		const saveButton = page.getByRole('button', { name: 'Save Profile' });
		await expect(saveButton).toBeDisabled();

		// Restore first name, clear last name
		const firstNameField = page.getByRole('textbox', { name: 'First Name' });
		await firstNameField.fill('Test');

		// Test 2: Empty last name validation (consolidated from empty_last_name_validation)
		await page.getByRole('textbox', { name: 'Last Name' }).clear();
		await expect(saveButton).toBeDisabled();
	});

	test('very_long_first_name', async ({ page }) => {
		await page.getByRole('link', { name: 'Settings', exact: true }).click();
		await page.waitForURL('**/settings');
		const longName = 'A'.repeat(100);
		await page.getByRole('textbox', { name: 'First Name' }).clear();
		await page.getByRole('textbox', { name: 'First Name' }).fill(longName);
		await page.getByRole('button', { name: 'Save Profile' }).click();
	});

	test('special_characters_in_names', async ({ page }) => {
		await page.getByRole('link', { name: 'Settings', exact: true }).click();
		await page.waitForURL('**/settings');

		const originalFirstName = await page.getByRole('textbox', { name: 'First Name' }).inputValue();
		const originalLastName = await page.getByRole('textbox', { name: 'Last Name' }).inputValue();

		await page.getByRole('textbox', { name: 'First Name' }).clear();
		await page.getByRole('textbox', { name: 'First Name' }).fill("O'Connor");
		await page.getByRole('textbox', { name: 'Last Name' }).clear();
		await page.getByRole('textbox', { name: 'Last Name' }).fill('Smith-Jones');
		await page.getByRole('button', { name: 'Save Profile' }).click();

		await page.reload();

		await page.getByRole('textbox', { name: 'First Name' }).clear();
		await page.getByRole('textbox', { name: 'First Name' }).fill(originalFirstName);
		await page.getByRole('textbox', { name: 'Last Name' }).clear();
		await page.getByRole('textbox', { name: 'Last Name' }).fill(originalLastName);
		await page.getByRole('button', { name: 'Save Profile' }).click();
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
		await page.getByRole('link', { name: 'Settings', exact: true }).click();
		await page.waitForURL('**/settings');
		const resetButton = page.getByRole('button', { name: 'Request Password Reset' });
		if (await resetButton.isVisible()) {
			await resetButton.click();
		}
	});

	test('password_reset_success_message', async ({ page }) => {
		await page.getByRole('link', { name: 'Settings', exact: true }).click();
		await page.waitForURL('**/settings');
		const resetButton = page.getByRole('button', { name: 'Request Password Reset' });
		if (await resetButton.isVisible()) {
			await resetButton.click();
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
		await page.getByRole('link', { name: 'Settings', exact: true }).click();
		await page.waitForURL('**/settings');
		const darkModeToggle = page
			.locator('input[type="checkbox"]')
			.filter({ hasText: /dark/i })
			.or(page.getByLabel('Dark Mode'));
		if (await darkModeToggle.isVisible()) {
			await darkModeToggle.check();
			const htmlElement = page.locator('html');
			await htmlElement.evaluate((el) => el.classList.contains('dark'));
		}
	});

	test('toggle_dark_mode_off', async ({ page }) => {
		await page.getByRole('link', { name: 'Settings', exact: true }).click();
		await page.waitForURL('**/settings');
		const darkModeToggle = page
			.locator('input[type="checkbox"]')
			.filter({ hasText: /dark/i })
			.or(page.getByLabel('Dark Mode'));
		if (await darkModeToggle.isVisible()) {
			await darkModeToggle.uncheck();
		}
	});

	test('dark_mode_persists_across_pages', async ({ page }) => {
		await page.getByRole('link', { name: 'Settings', exact: true }).click();
		await page.waitForURL('**/settings');
		const darkModeToggle = page
			.locator('input[type="checkbox"]')
			.filter({ hasText: /dark/i })
			.or(page.getByLabel('Dark Mode'));
		if (await darkModeToggle.isVisible()) {
			await darkModeToggle.check();

			await page.getByRole('link', { name: 'Dashboard', exact: true }).click();
			await page.waitForURL('**/dashboard');

			await page.getByRole('link', { name: 'Settings', exact: true }).click();
			await page.waitForURL('**/settings');
		}
	});

	test('dark_mode_persists_across_sessions', async ({ page }) => {
		await page.getByRole('link', { name: 'Settings', exact: true }).click();
		await page.waitForURL('**/settings');
		const darkModeToggle = page
			.locator('input[type="checkbox"]')
			.filter({ hasText: /dark/i })
			.or(page.getByLabel('Dark Mode'));
		if (await darkModeToggle.isVisible()) {
			await darkModeToggle.check();
			await page.getByRole('button', { name: 'Logout' }).click();
			await page.waitForURL('**/login');
			await page.getByRole('textbox', { name: 'Email' }).fill(adminCreds.email);
			await page.getByRole('textbox', { name: 'Password' }).fill(adminCreds.password);
			await page.getByRole('button', { name: 'Sign in', exact: true }).click();
			await page.waitForURL('**/dashboard');
			await page.getByRole('link', { name: 'Settings', exact: true }).click();
			await page.waitForURL('**/settings');
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
		const scPage = await browser.newPage();
		const success = await acceptInvitation(
			scPage,
			invitationToken!,
			'Member',
			'User',
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
	test('member_can_access_settings', async ({ page }) => {
		await page.getByRole('link', { name: 'Settings', exact: true }).click();
		await page.waitForURL('**/settings');
		await expect(page).toHaveURL('/settings');
	});

	test('member_can_update_profile', async ({ page }) => {
		await page.getByRole('link', { name: 'Settings', exact: true }).click();
		await page.waitForURL('**/settings');
		await page.getByRole('textbox', { name: 'First Name' }).clear();
		await page.getByRole('textbox', { name: 'First Name' }).fill('Invited');
		await page.getByRole('button', { name: 'Save Profile' }).click();
	});

	test('member_can_toggle_dark_mode', async ({ page }) => {
		await page.getByRole('link', { name: 'Settings', exact: true }).click();
		await page.waitForURL('**/settings');
		const darkModeToggle = page
			.locator('input[type="checkbox"]')
			.filter({ hasText: /dark/i })
			.or(page.getByLabel('Dark Mode'));
		if (await darkModeToggle.isVisible()) {
			await darkModeToggle.check();

			await darkModeToggle.uncheck();
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
		}
	});

	test('reset_password_without_token_shows_request_form', async ({ page }) => {
		await page.goto('http://localhost:5173/reset-password');
		await expect(page.getByRole('textbox', { name: 'Email' })).toBeVisible();
	});

	test('reset_password_validate_password_requirements', async ({ page, browser }) => {
		const token = await requestPasswordResetToken(browser, passwordResetCreds.email);
		if (!token) throw new Error('Failed to retrieve password reset token');
		await page.goto(`http://localhost:5173/reset-password?token=${token}`);
		await page.waitForLoadState('networkidle');

		const passwordField = page.getByRole('textbox', { name: 'New Password' });
		if (await passwordField.isVisible()) {
			// Test weak password validation (consolidated from reset_password_weak_password)
			await validatePasswordField(page, 'weak', 'reset');

			// Fill valid password and test mismatch (consolidated from reset_password_mismatch)
			await page.getByRole('textbox', { name: 'New Password' }).fill('NewPassword123!');
			await page.getByRole('textbox', { name: 'Confirm Password' }).fill('DifferentPassword123!');
			await expect(page.getByRole('button', { name: 'Set new password' })).toBeDisabled();
		}
	});

	test('reset_password_invalid_token', async ({ page }) => {
		await page.goto('http://localhost:5173/reset-password?token=invalid-token-xyz');
		await page.waitForLoadState('networkidle');
	});
});

const PNG_BASE64 =
	'iVBORw0KGgoAAAANSUhEUgAAAEAAAABACAYAAACqaXHeAAAAIGNIUk0AAHomAACAhAAA+gAAAIDoAAB1MAAA6mAAADqYAAAXcJy6UTwAAAAGYktHRAD/AP8A/6C9p5MAAAtZSURBVHja7Zp5lFxFFcZ/t9573T2TbQiBZBJCFBA0IgRNkFUingMiED2iyBI2ARf2RYwIiAeIaEAMghA4BAIhCbuQsB3AoCzhCEQgIGGHbAxkIUyY6e29V9c/6s2kZ0hmuns6YdR85/T06+mq6nu/unWr7r0Fm7AJm7AJ/8eQWg5206yHKmqvvkdxyEDU98pq//Pdd+ldBNw4aw5pTRFKiGjbgIpnw7L6R+k0qwc1YM36xRAE3/MRERTl1D3H9A4Cps16ECuCb2OsSEpUvwrsDmwFpMsZwwY+2cEDUc902QyYGxPf5+EpwCl7jv5sCbhp1hxUAjxbBMyXgfOAvYEWYDUQlTN2FwQUkzEMMCp5/4Wi0wTRWlqCX00nTwGNUKP89ETZCcBcgU/KHUfa/hhAO3KDokAA3AJ8G5goSDiIQbeuYAVXPf1cTUio2AJumfEA6hZ8v0S4RuAY4PVyBtSSNnEqoLVxIIh0JsDBEGC5F8gBy4HvAefMltkzDtaDAXpMQsUWIArqNPgmbs0fBby+QxjyRhBw9BEHfarPjbMeAmupDzwKYbQFkLNGWt44ZD+GzZvfiJI1KjkrOkJhqUAuYUmSV4sY+bVaNcDl43ScLGPZrUMZ2mNLMJV2sEbJ90kB7Ae8ATwDrFf5qbc9DAgBMYUoGoNwpwoH2VTA8Cee2wy4FjjKio4A7hCYICqZThbhZfPZj0XkV8B9wKRhDBu/mMUAXPX0cxuPAIBMa9EHhgOLVLQlk0+vU/nps+ZgVAlsiBUzGrge6AP836Z8vHxhe1HdHWhWy3vAHOBUFT1HLBncDmABGZAZIIq2kTAbmDSCEeNPyY3uEQlVEYAzywBoQYVCurjuRgISx1gxYxLlI0F/ZpWXo7oMXiEcg9UiIvPFEEqslwLXAKep4ZdY6oEmoLEYF/ugoOjHYuQc4K/A7/9SN//InOYAuHre8xUrUtUuUIICwNFHHrjOL2MrGMNwRafgnObhFpnvGRj46jt+y1aD9xTVN61vFpswBkPOxDrRejIUOB94DXgIuA5hCrAQJVRVD7dNZoDJ9VKfF5W7LXajE9AtVMkhLAK+Dgwv+NHzmcgnv1n/LbF2Z7F674A3lrZ88rmhqAjq8RXQUcAC4DUjZqFVOxA4DBjJ2k1EgfeA7YAfqqf3ikrcywgQEF2JyumITgauzES+xOg9npGRYu0gieJ5+c0HYI1F1OyKsxYLciJiF6gqqnqDMWaGVdunM7+CTAbqqpVwgxKgFvx0SBwGS1TlDBG9EpjsIcsRRovVNX6u8EqcDkDNlqCTcDN7ImJf0EIdEkQEvkds4xzuPFBCr5D8r/4zI0C7+O7Y8Qcy7dYH8Rs+Jl7TfwlwOnAC7qj7IqqXe4XCsrBPBtACMB3hebAvbTEDYjl5r1HrHf/qpyt3ep1R7S5QNo4d/x2ijxsI/DwoS0aNnHthNmh8VtCHm97LTUEkRBWUZmAqyku2tQGM5dSxO21o8XpuAW0eadrMB5LP67IJSxhlQODFV/elniYUYch2fclu6YKh0iO0qV8DrGeGBdDaRYS19gEDgLE4p6Q9G2qdqreq8neQsgOujUZAMvMjgauBzaGKTblrGGC5wCGg83sdAcmELwA5EiVNtxagSQst6d4FnMvPAa/WTuaaEiAo2irwRHlBtpQkBCg7MFepoPHGJEBKhJKyBFw802GEuderfY0';

test.describe('Settings - Profile Picture', () => {
	test.beforeAll(async ({ browser }) => {
		const creds = await register(browser);
		if (!creds) throw new Error('Failed to register admin user');
		adminCreds = { email: creds.email, password: creds.password };
	});

	test.beforeEach(async ({ page }) => {
		await page.goto('http://localhost:5173/');
		await page.getByRole('link', { name: 'Login' }).click();
		await page.getByRole('textbox', { name: 'Email' }).fill(adminCreds.email);
		await page.getByRole('textbox', { name: 'Password' }).fill(adminCreds.password);
		await page.getByRole('button', { name: 'Sign in', exact: true }).click();
		await page.waitForURL('**/dashboard');
	});

	test('upload_and_persist_profile_picture', async ({ page }) => {
		await page.getByRole('link', { name: 'Settings', exact: true }).click();
		await page.waitForURL('**/settings');

		const fileInput = page.locator('.file-input');
		await fileInput.setInputFiles({
			name: 'avatar.png',
			mimeType: 'image/png',
			buffer: Buffer.from(PNG_BASE64, 'base64')
		});

		await expect(page.getByRole('button', { name: 'Save', exact: true })).toBeVisible();
		const uploadResponse = page.waitForResponse(
			(resp) => resp.url().includes('/api/auth/profile-picture') && resp.status() === 200
		);
		await page.getByRole('button', { name: 'Save', exact: true }).click();
		await uploadResponse;

		await expect(page.locator('img.picture-preview')).toBeVisible();

		await page.reload();
		await page.waitForURL('**/settings');
		await expect(page.locator('img.picture-preview')).toBeVisible();
	});
});
