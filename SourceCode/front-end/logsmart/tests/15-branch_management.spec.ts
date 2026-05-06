import { test, expect } from '@playwright/test';
import { register } from './utils';

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
	await expect(adminCreds.page!.getByText('testname1')).toBeVisible();
	await expect(adminCreds.page!.getByText('London Office')).toBeVisible();

	await adminCreds.page!.close();
});
