import { test, expect } from '@playwright/test';
import { register, sendInvitationOnPage, acceptInvitation } from './utils';

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
