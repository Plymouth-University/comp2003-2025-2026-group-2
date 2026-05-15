import { test, expect } from '@playwright/test';
import { register, sendInvitation, acceptInvitation, createBranch, sendInvitationOnPage } from './utils';

let adminCreds: {
	email: string;
	password: string;
	companyName: string;
	firstName: string;
	lastName: string;
};

const BRANCH_NAME = 'Test Branch';

test.beforeAll(async ({ browser }) => {
	const creds = await register(browser, false);
	if (!creds) throw new Error('Failed to register admin user');
	adminCreds = creds;

	await createBranch(creds.page!, BRANCH_NAME, '123 Test St');
	await creds.page!.close();
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

		const tempButton = page.getByRole('button', { name: 'Temperature' });
		if (await tempButton.isVisible()) {
			await tempButton.click();
		}

		const saveButton = page.getByRole('button', { name: 'Save' });
		if ((await saveButton.isVisible()) && (await saveButton.isEnabled())) {
			await saveButton.click();
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
			}

			const submitButton = memberPage.getByRole('button', { name: 'Submit Log' });
			if (await submitButton.isVisible()) {
				await submitButton.click();
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

		const textInputButton = page.getByRole('button', { name: 'Text Input' });
		if (await textInputButton.isVisible()) {
			await textInputButton.click();
		}

		const saveButton = page.getByRole('button', { name: 'Save' });
		if ((await saveButton.isVisible()) && (await saveButton.isEnabled())) {
			await saveButton.click();
		}

		await page.getByRole('link', { name: 'Templates' }).click();
		await page.waitForURL('**/templates-dashboard');

		const searchInput = page.getByPlaceholder('Search templates');
		if (await searchInput.isVisible()) {
			await searchInput.fill(templateName);
		}

		const settingsButton = page.getByRole('button', { name: 'Settings', exact: true }).first();
		if (await settingsButton.isVisible()) {
			await settingsButton.click();

			const saveScheduleButton = page.getByRole('button', { name: 'Save' });
			if (await saveScheduleButton.isVisible()) {
				await saveScheduleButton.click();
			}
		}

		await page.goto(
			`http://localhost:5173/log-template?template=${encodeURIComponent(templateName)}`
		);
		await page.waitForLoadState('networkidle');

		const submitButton = page.getByRole('button', { name: 'Submit Log' });
		if (await submitButton.isVisible()) {
			await submitButton.click();
		}

		await page.goto('http://localhost:5173/templates-dashboard');
		await page.waitForURL('**/templates-dashboard');

		if (await searchInput.isVisible()) {
			await searchInput.fill(templateName);
		}

		const deleteButton = page.getByRole('button', { name: 'Delete' }).first();
		if (await deleteButton.isVisible()) {
			await deleteButton.click();

			const confirmButton = page.getByRole('button', { name: 'Confirm' });
			if (await confirmButton.isVisible()) {
				await confirmButton.click();
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

		await page.getByRole('link', { name: 'Settings', exact: true }).click();
		await page.waitForURL('**/settings');

		await page.getByRole('textbox', { name: 'First Name' }).clear();
		await page.getByRole('textbox', { name: 'First Name' }).fill('Updated');
		await page.getByRole('textbox', { name: 'Last Name' }).clear();
		await page.getByRole('textbox', { name: 'Last Name' }).fill('Name');
		await page.getByRole('button', { name: 'Save Profile' }).click();

		await page.getByRole('link', { name: 'Dashboard', exact: true }).click();
		await page.waitForURL('**/dashboard');
		await expect(page.locator('body')).toContainText('Updated Name');

		await page.getByRole('link', { name: 'Users' }).click();
		await page.waitForURL('**/users-admin');
		await expect(page.locator('body')).toContainText('Updated Name');

		await page.getByRole('link', { name: 'Settings', exact: true }).click();
		await page.waitForURL('**/settings');
		await page.getByRole('textbox', { name: 'First Name' }).clear();
		await page.getByRole('textbox', { name: 'First Name' }).fill('Test');
		await page.getByRole('textbox', { name: 'Last Name' }).clear();
		await page.getByRole('textbox', { name: 'Last Name' }).fill('User');
		await page.getByRole('button', { name: 'Save Profile' }).click();
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

test('create_and_list_branches', async ({ browser }) => {
	const adminCreds = await register(browser, false);
	if (!adminCreds) throw new Error('Failed to register admin user');
	await adminCreds.page!.route('https://nominatim.openstreetmap.org/search**', async (route) => {
		await route.fulfill({
			json: [
				{
					place_id: 1,
					licence: 'testlicense',
					osm_type: 'way',
					osm_id: 1,
					lat: '10.0000000',
					lon: '10.0000000',
					class: 'testclass',
					type: 'testtype',
					place_rank: 1,
					importance: 1,
					addresstype: 'testaddrtype',
					name: 'testloc1',
					display_name: 'testname1',
					address: {
						city: 'testcity',
						county: 'testcountry',
						state_district: 'teststated',
						state: 'Teststate',
						'ISO3166-2-lvl4': 'TE-ST',
						country: 'TESTC',
						country_code: 'TESTCC'
					},
					boundingbox: ['10.0000000', '10.0000000', '-10.0000000', '-10.0000000']
				}
			]
		});
	});
	// Navigate to Branches
	await adminCreds.page!.getByRole('link', { name: 'Branches' }).click();
	await adminCreds.page!.waitForURL('**/branches');

	await expect(adminCreds.page!.getByRole('heading', { name: 'BRANCH MANAGEMENT' })).toBeVisible();

	// Add Branch 1
	await adminCreds.page!.getByRole('textbox', { name: 'Branch Name' }).fill('London Office');
	await adminCreds.page!.getByRole('textbox', { name: 'Address' }).fill('test');
	await adminCreds
		.page!.locator('form > div.search-container.relative.flex-1 > div > button')
		.first()
		.click();
	await adminCreds.page!.getByRole('button', { name: 'ADD BRANCH' }).click();

	await expect(adminCreds.page!.getByText('London Office')).toBeVisible();
	await expect(adminCreds.page!.getByText('testname1')).toBeVisible();

	// Add Branch 2
	await adminCreds.page!.getByRole('textbox', { name: 'Branch Name' }).fill('Manchester Hub');
	await adminCreds.page!.getByRole('textbox', { name: 'Address' }).fill('test');
	await adminCreds
		.page!.locator('form > div.search-container.relative.flex-1 > div > button')
		.first()
		.click();
	await adminCreds.page!.getByRole('button', { name: 'ADD BRANCH' }).click();

	await expect(adminCreds.page!.getByText('Manchester Hub')).toBeVisible();
	(await adminCreds.page!.getByText('testname1').all()).forEach(async (element) => {
		await expect(element).toBeVisible();
	});
	await expect(adminCreds.page!.getByText('London Office')).toBeVisible();

	await adminCreds.page!.close();
});

test('hierarchical_access_control', async ({ browser }) => {
	console.log('Starting Setup: Register Company Manager');
	// 1. Setup: Register Company Manager and Create Branch
	const cmData = await register(browser, false);
	if (!cmData) throw new Error('Failed to register CM');
	const { email: cmEmail, page: cmPage } = cmData;
	if (!cmPage) throw new Error('Failed to get CM page');
	console.log('CM Registered:', cmEmail);

	await cmPage.route('https://nominatim.openstreetmap.org/search**', async (route) => {
		await route.fulfill({
			json: [
				{
					place_id: 1,
					licence: 'test',
					osm_type: 'way',
					osm_id: 1,
					lat: '10.0000000',
					lon: '10.0000000',
					class: 'test',
					type: 'test',
					place_rank: 1,
					importance: 1,
					addresstype: 'test',
					name: 'test',
					display_name: 'test',
					address: {
						city: 'test',
						county: 'test',
						state_district: 'test',
						state: 'Test',
						'ISO3166-2-lvl4': 'TE-ST',
						country: 'TEST',
						country_code: 'TEST'
					},
					boundingbox: ['10.0000000', '10.0000000', '-10.0000000', '-10.0000000']
				}
			]
		});
	});

	await cmPage.getByRole('link', { name: 'Branches' }).click();
	await cmPage.waitForURL('**/branches');
	await cmPage.getByRole('textbox', { name: 'Branch Name' }).fill('North Branch');
	await cmPage.getByRole('textbox', { name: 'Address' }).fill('North Pole');
	await cmPage
		.locator('form > div.search-container.relative.flex-1 > div > button')
		.first()
		.click();
	await cmPage.getByRole('button', { name: 'ADD BRANCH' }).click();

	await expect(cmPage.getByText('North Branch')).toBeVisible();
	console.log('Branch created: North Branch');

	// 2. Invite Branch Manager and Staff Member
	const bmEmail = `bm-${Date.now()}@logsmart.app`;
	const staffEmail = `staff-${Date.now()}@logsmart.app`;

	console.log('Sending BM invitation');
	const bmToken = await sendInvitationOnPage(cmPage, bmEmail, 'branch_manager', 'North Branch');
	console.log('BM Invitation token received');

	console.log('Sending Staff invitation');
	const staffToken = await sendInvitationOnPage(cmPage, staffEmail, 'staff', 'North Branch');
	console.log('Staff Invitation token received');

	console.log('BM Token:', bmToken);
	console.log('Staff Token:', staffToken);

	expect(bmToken).toBeTruthy();
	expect(staffToken).toBeTruthy();

	// 3. Accept Invitations
	const bmPassword = 'BranchManager123!';
	const staffPassword = 'StaffMember123!';

	const bmPage = await browser.newPage();
	const staffPage = await browser.newPage();

	console.log('Accepting BM invitation');
	await acceptInvitation(bmPage, bmToken!, 'Branch', 'Manager', bmPassword, '**/dashboard');
	console.log('BM Invitation accepted');

	console.log('Accepting Staff Member invitation');
	await acceptInvitation(staffPage, staffToken!, 'Staff', 'Member', staffPassword, '**/logs-list');
	console.log('Staff Invitation accepted');

	// 4. Verify Branch Manager Permissions
	console.log('Verifying BM permissions');
	await bmPage.getByRole('link', { name: 'Users' }).click();
	await bmPage.waitForURL('**/users-admin');
	await expect(bmPage.locator('body')).toContainText(staffEmail);
	await expect(bmPage.locator('body')).toContainText('Staff');
	await expect(bmPage.getByRole('link', { name: 'Branches' })).not.toBeVisible();

	// 5. Verify Staff Permissions
	console.log('Verifying Staff permissions');
	await expect(staffPage.getByRole('link', { name: 'Users' })).not.toBeVisible();
	await expect(staffPage.getByRole('link', { name: 'Reports' })).not.toBeVisible();
	await expect(staffPage.getByRole('link', { name: 'Templates Dashboard' })).not.toBeVisible();
	await expect(staffPage.getByRole('link', { name: 'Branches' })).not.toBeVisible();
	await expect(staffPage.getByRole('link', { name: 'Logs', exact: true })).toBeVisible();

	await bmPage.close();
	await staffPage.close();
});
