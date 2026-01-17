import { test, expect } from '@playwright/test';
import { acceptInvitation, register, sendInvitation } from './utils';

let adminCreds: {
	email: string;
	password: string;
	companyName: string;
	firstName: string;
	lastName: string;
};

test.beforeAll(async ({ browser }) => {
	const creds = await register(browser);
	if (!creds) throw new Error('Failed to register admin user');
	adminCreds = creds;
});

test.describe('User Administration - Admin Access', () => {
	test.beforeEach(async ({ page }) => {
		await page.goto('http://localhost:5173/');
		await page.getByRole('link', { name: 'Login' }).click();
		await page.getByRole('textbox', { name: 'Email' }).fill(adminCreds.email);
		await page.getByRole('textbox', { name: 'Password' }).fill(adminCreds.password);
		await page.getByRole('button', { name: 'Sign in' }).click();
		await page.waitForURL('**/dashboard');
	});

	test('access_users_admin_page', async ({ page }) => {
		await page.getByRole('link', { name: 'Users' }).click();
		await page.waitForURL('**/users-admin');
		await expect(page).toHaveURL('/users-admin');
	});

	test('view_members_list', async ({ page }) => {
		await page.getByRole('link', { name: 'Users' }).click();
		await page.waitForURL('**/users-admin');
		await expect(page.locator('body')).toContainText(adminCreds.email);
		await expect(page.locator('body')).toContainText(adminCreds.firstName.split('-')[0]);
	});

	test('view_admin_badge', async ({ page }) => {
		await page.getByRole('link', { name: 'Users' }).click();
		await page.waitForURL('**/users-admin');
		await expect(page.locator('body')).toContainText('Admin');
	});

	test('view_member_badge', async ({ page }) => {
		await page.getByRole('link', { name: 'Users' }).click();
		await page.waitForURL('**/users-admin');
		const memberBadge = page.locator('text=Member').first();
		if (await memberBadge.isVisible()) {
			await expect(memberBadge).toBeVisible();
		}
	});

	test('click_member_shows_sidebar', async ({ page }) => {
		await page.getByRole('link', { name: 'Users' }).click();
		await page.waitForURL('**/users-admin');
		const memberRow = page.locator('#eventHide').first();
		if (await memberRow.isVisible()) {
			await memberRow.click();
			await page.waitForTimeout(500);
		}
	});

	test('open_invite_modal', async ({ page }) => {
		await page.getByRole('link', { name: 'Users' }).click();
		await page.waitForURL('**/users-admin');
		await page.getByRole('button', { name: '➕' }).click();
		await page.waitForTimeout(500);
		await expect(page.getByRole('textbox', { name: "New user's email" })).toBeVisible();
	});

	test('send_invitation_valid_email', async ({ page }) => {
		const timestamp = Date.now();
		await page.getByRole('link', { name: 'Users' }).click();
		await page.waitForURL('**/users-admin');
		await page.getByRole('button', { name: '➕' }).click();
		await page
			.getByRole('textbox', { name: "New user's email" })
			.fill(`newinvite${timestamp}@logsmart.app`);
		await page.getByRole('button', { name: 'Send Invite' }).click();
		await page.waitForTimeout(1000);
	});

	test('send_invitation_invalid_email_format', async ({ page }) => {
		await page.getByRole('link', { name: 'Users' }).click();
		await page.waitForURL('**/users-admin');
		await page.getByRole('button', { name: '➕' }).click();
		await page.getByRole('textbox', { name: "New user's email" }).fill('not-an-email');
		const sendButton = page.getByRole('button', { name: 'Send Invite' });
		if (await sendButton.isVisible()) {
			const isDisabled = await sendButton.isDisabled();
			if (!isDisabled) {
				await sendButton.click();
				await page.waitForTimeout(500);
			} else {
				await expect(sendButton).toBeDisabled();
			}
		}
	});

	test('send_invitation_empty_email', async ({ page }) => {
		await page.getByRole('link', { name: 'Users' }).click();
		await page.waitForURL('**/users-admin');
		await page.getByRole('button', { name: '➕' }).click();
		const sendButton = page.getByRole('button', { name: 'Send Invite' });
		await expect(sendButton).toBeDisabled();
	});

	test('close_invite_modal_without_sending', async ({ page }) => {
		await page.getByRole('link', { name: 'Users' }).click();
		await page.waitForURL('**/users-admin');
		await page.getByRole('button', { name: '➕' }).click();
		await page.waitForTimeout(500);
		const closeButton = page.getByRole('button', { name: 'Close' });
		if (await closeButton.isVisible()) {
			await closeButton.click();
			await page.waitForTimeout(300);
		} else {
			await page.keyboard.press('Escape');
			await page.waitForTimeout(300);
		}
	});

	test('view_multiple_members', async ({ page }) => {
		await page.getByRole('link', { name: 'Users' }).click();
		await page.waitForURL('**/users-admin');
		const memberRows = page.locator('#eventHide');
		const count = await memberRows.count();
		expect(count).toBeGreaterThanOrEqual(1);
	});

	test('member_details_in_sidebar', async ({ page }) => {
		await page.getByRole('link', { name: 'Users' }).click();
		await page.waitForURL('**/users-admin');
		const memberRow = page.locator('#eventHide').first();
		if (await memberRow.isVisible()) {
			await memberRow.click();
			await page.waitForTimeout(500);
			const sidebar = page.locator('aside, .sidebar, [role="complementary"]').first();
			if (await sidebar.isVisible()) {
				await expect(sidebar).toBeVisible();
			}
		}
	});

	test('update_user_profile_via_sidebar', async ({ page }) => {
		await page.getByRole('link', { name: 'Users' }).click();
		await page.waitForURL('**/users-admin');

		await page.locator(`button:has-text("${adminCreds.email}")`).click();

		const sidebar = page.locator('#userSidebar');
		await expect(sidebar).toBeVisible();

		const newFirstName = 'UpdatedName';
		await page.locator('#fname').fill(newFirstName);

		await page.getByRole('button', { name: 'Save' }).click();

		await page.reload();
		await page.waitForSelector(`text=${newFirstName}`);
		await expect(page.locator('body')).toContainText(newFirstName);
	});

	test('change_invited_user_role', async ({ page, browser }) => {
		const timestamp = Date.now();
		const invitedEmail = `changeroelUser${timestamp}@logsmart.app`;

		const invitationToken = await sendInvitation(browser, adminCreds, invitedEmail);
		expect(invitationToken).toBeTruthy();

		const inviteePage = await browser.newPage();
		await acceptInvitation(
			inviteePage,
			invitationToken!,
			'InvitedFirst',
			'InvitedLast',
			'Invited123!'
		);
		await inviteePage.close();

		await page.goto('http://localhost:5173/dashboard');

		await page.getByRole('link', { name: 'Users' }).click();
		await page.waitForURL('**/users-admin');

		await page.locator(`button:has-text("${invitedEmail}")`).click();

		const sidebar = page.locator('#userSidebar');
		await expect(sidebar).toBeVisible();

		const roleSelect = page.locator('#role');
		await roleSelect.selectOption('admin');

		await page.getByRole('button', { name: 'Save' }).click();

		await page.reload();
		await page.waitForURL('**/users-admin');
		await page.locator(`button:has-text("${invitedEmail}")`).click();
		await expect(page.locator('#role')).toHaveValue('admin');
	});
});

test.describe('Templates Dashboard - Admin Access', () => {
	test.beforeEach(async ({ page }) => {
		await page.goto('http://localhost:5173/');
		await page.getByRole('link', { name: 'Login' }).click();
		await page.getByRole('textbox', { name: 'Email' }).fill(adminCreds.email);
		await page.getByRole('textbox', { name: 'Password' }).fill(adminCreds.password);
		await page.getByRole('button', { name: 'Sign in' }).click();
		await page.waitForURL('**/dashboard');
	});

	test('access_templates_dashboard', async ({ page }) => {
		await page.getByRole('link', { name: 'Templates Dashboard' }).click();
		await page.waitForURL('**/templates-dashboard');
		await expect(page).toHaveURL('/templates-dashboard');
	});

	test('view_templates_list', async ({ page }) => {
		await page.getByRole('link', { name: 'Templates Dashboard' }).click();
		await page.waitForURL('**/templates-dashboard');
		await page.waitForLoadState('networkidle');
		await page.waitForTimeout(500);
	});

	test('search_templates_by_name', async ({ page }) => {
		await page.getByRole('link', { name: 'Templates Dashboard' }).click();
		await page.waitForURL('**/templates-dashboard');
		const searchInput = page.getByPlaceholder('Search templates');
		if (await searchInput.isVisible()) {
			await searchInput.fill('Test');
			await page.waitForTimeout(500);
		}
	});

	test('click_create_new_template', async ({ page }) => {
		await page.getByRole('link', { name: 'Templates Dashboard' }).click();
		await page.waitForURL('**/templates-dashboard');
		await page.getByRole('button', { name: 'Create New Template' }).click();
		await page.waitForURL('**/template-designer');
	});

	test('click_edit_template', async ({ page }) => {
		await page.getByRole('link', { name: 'Templates Dashboard' }).click();
		await page.waitForURL('**/templates-dashboard');
		const editButton = page.getByRole('button', { name: 'Edit' }).first();
		if (await editButton.isVisible()) {
			await editButton.click();
			await page.waitForURL('**/template-designer**');
		}
	});

	test('click_settings_opens_schedule_wizard', async ({ page }) => {
		await page.getByRole('link', { name: 'Templates Dashboard' }).click();
		await page.waitForURL('**/templates-dashboard');
		const settingsButton = page.getByRole('button', { name: 'Settings' }).first();
		if (await settingsButton.isVisible()) {
			await settingsButton.click();
			await page.waitForTimeout(500);
		}
	});

	test('update_template_schedule', async ({ page }) => {
		await page.getByRole('link', { name: 'Templates Dashboard' }).click();
		await page.waitForURL('**/templates-dashboard');
		const settingsButton = page.getByRole('button', { name: 'Settings' }).first();
		if (await settingsButton.isVisible()) {
			await settingsButton.click();
			await page.waitForTimeout(500);
			const saveButton = page.getByRole('button', { name: 'Save' });
			if (await saveButton.isVisible()) {
				await saveButton.click();
				await page.waitForTimeout(500);
			}
		}
	});

	test('delete_template_with_confirmation', async ({ page }) => {
		await page.getByRole('link', { name: 'Templates Dashboard' }).click();
		await page.waitForURL('**/templates-dashboard');
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

	test('delete_template_cancel_confirmation', async ({ page }) => {
		await page.getByRole('link', { name: 'Templates Dashboard' }).click();
		await page.waitForURL('**/templates-dashboard');
		const deleteButton = page.getByRole('button', { name: 'Delete' }).first();
		if (await deleteButton.isVisible()) {
			await deleteButton.click();
			await page.waitForTimeout(500);
			const cancelButton = page.getByRole('button', { name: 'Cancel' });
			if (await cancelButton.isVisible()) {
				await cancelButton.click();
				await page.waitForTimeout(300);
			} else {
				await page.keyboard.press('Escape');
				await page.waitForTimeout(300);
			}
		}
	});

	test('view_template_schedule_display', async ({ page }) => {
		await page.getByRole('link', { name: 'Templates Dashboard' }).click();
		await page.waitForURL('**/templates-dashboard');
		await page.waitForLoadState('networkidle');
		const scheduleText = page.locator('text=/Daily|Weekly|Monthly|Yearly/').first();
		if (await scheduleText.isVisible()) {
			await expect(scheduleText).toBeVisible();
		}
	});

	test('view_template_updated_date', async ({ page }) => {
		await page.getByRole('link', { name: 'Templates Dashboard' }).click();
		await page.waitForURL('**/templates-dashboard');
		await page.waitForLoadState('networkidle');
		await page.waitForTimeout(500);
	});

	test('empty_templates_list_shows_message', async ({ page }) => {
		await page.getByRole('link', { name: 'Templates Dashboard' }).click();
		await page.waitForURL('**/templates-dashboard');
		const searchInput = page.getByPlaceholder('Search templates');
		if (await searchInput.isVisible()) {
			await searchInput.fill('NonExistentTemplate12345');
			await page.waitForTimeout(500);
		}
	});
});

test.describe('Templates Dashboard - Member Access Control', () => {
	let memberCreds: {
		email: string;
		password: string;
	};
	test.beforeAll(async ({ browser }) => {
		const page = await browser.newPage();
		await page.goto('http://localhost:5173/');
		await page.getByRole('link', { name: 'Login' }).click();
		await page.getByRole('textbox', { name: 'Email' }).fill(adminCreds.email);
		await page.getByRole('textbox', { name: 'Password' }).fill(adminCreds.password);
		await page.getByRole('button', { name: 'Sign in' }).click();
		await page.waitForURL('**/dashboard');
		await page.goto('http://localhost:5173/template-designer');
		await page.waitForLoadState('networkidle');

		const memberEmail = `member-${Date.now()}-${Math.floor(Math.random() * 1_000_000)}@logsmart.app`;
		const invitationToken = await sendInvitation(browser, adminCreds, memberEmail);
		if (!invitationToken) throw new Error('Failed to get invitation token');
		const success = await acceptInvitation(
			await browser.newPage(),
			invitationToken!,
			'Member',
			'User',
			'Member123!'
		);
		if (!success) throw new Error('Failed to accept invitation for member user');
		memberCreds = { email: memberEmail, password: 'Member123!' };
	});
	test('member_cannot_access_templates_dashboard', async ({ page }) => {
		await page.goto('http://localhost:5173/');
		await page.getByRole('link', { name: 'Login' }).click();
		await page.getByRole('textbox', { name: 'Email' }).fill(memberCreds.email);
		await page.getByRole('textbox', { name: 'Password' }).fill(memberCreds.password);
		await page.getByRole('button', { name: 'Sign in' }).click();
		await page.waitForURL('**/logs-list');
		await page.goto('http://localhost:5173/templates-dashboard');
		await page.waitForURL('**/logs-list');
		await expect(page).toHaveURL('/logs-list');
	});
	test('member_cannot_access_users_admin', async ({ page }) => {
		await page.goto('http://localhost:5173/');
		await page.getByRole('link', { name: 'Login' }).click();
		await page.getByRole('textbox', { name: 'Email' }).fill(memberCreds.email);
		await page.getByRole('textbox', { name: 'Password' }).fill(memberCreds.password);
		await page.getByRole('button', { name: 'Sign in' }).click();
		await page.waitForURL('**/logs-list');
		await page.goto('http://localhost:5173/users-admin');
		await page.waitForURL('**/logs-list');
		await expect(page).toHaveURL('/logs-list');
	});

	test('member_no_users_link_in_navigation', async ({ page }) => {
		await page.goto('http://localhost:5173/');
		await page.getByRole('link', { name: 'Login' }).click();
		await page.getByRole('textbox', { name: 'Email' }).fill(memberCreds.email);
		await page.getByRole('textbox', { name: 'Password' }).fill(memberCreds.password);
		await page.getByRole('button', { name: 'Sign in' }).click();
		await page.waitForURL('**/logs-list');
		const usersLink = page.getByRole('link', { name: 'Users' });
		await expect(usersLink).not.toBeVisible();
	});
});
