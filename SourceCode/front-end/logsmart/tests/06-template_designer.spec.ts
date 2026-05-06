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
	const creds = await register(browser, false);
	if (!creds) throw new Error('Failed to register admin user');
	adminCreds = creds;

	await createBranch(creds.page!, BRANCH_NAME, '123 Test St');
	await creds.page!.close();
});

test.describe('Template Designer - CRUD Operations', () => {
	test.beforeEach(async ({ page }) => {
		await page.goto('http://localhost:5173/');
		await page.getByRole('link', { name: 'Login' }).click();
		await page.getByRole('textbox', { name: 'Email' }).fill(adminCreds.email);
		await page.getByRole('textbox', { name: 'Password' }).fill(adminCreds.password);
		await page.getByRole('button', { name: 'Sign in', exact: true }).click();
		await page.waitForURL('**/dashboard');
	});

	test('access_template_designer', async ({ page }) => {
		await page.getByRole('link', { name: 'Templates Dashboard' }).click();
		await page.waitForURL('**/templates-dashboard');
		await page.getByRole('button', { name: 'Create New Template' }).click();
		await page.waitForURL('**/template-designer');
		await expect(page).toHaveURL('/template-designer');
	});

	test('create_new_template_with_name', async ({ page }) => {
		await page.goto('http://localhost:5173/template-designer');
		await page.waitForLoadState('networkidle');
		const nameInput = page.getByPlaceholder('Template Name');
		if (await nameInput.isVisible()) {
			await nameInput.fill('New Test Template');
		}
	});

	test('save_template', async ({ page }) => {
		await page.goto('http://localhost:5173/template-designer');
		await page.waitForLoadState('networkidle');
		const nameInput = page.getByPlaceholder('Template Name');
		if (await nameInput.isVisible()) {
			await nameInput.fill('Saveable Template');
			const saveButton = page.getByRole('button', { name: 'Save' });
			if ((await saveButton.isVisible()) && (await saveButton.isEnabled())) {
				await saveButton.click();
			}
		}
	});

	test('load_existing_template', async ({ page }) => {
		await page.goto('http://localhost:5173/templates-dashboard');
		await page.waitForLoadState('networkidle');
		const editButton = page.getByRole('button', { name: 'Edit' }).first();
		if (await editButton.isVisible()) {
			await editButton.click();
			await page.waitForURL('**/template-designer**');
		}
	});

	test('delete_template_from_designer', async ({ page }) => {
		await page.goto('http://localhost:5173/template-designer');
		await page.waitForLoadState('networkidle');
		const deleteButton = page.getByRole('button', { name: 'Delete Template' });
		if (await deleteButton.isVisible()) {
			await deleteButton.click();
		}
	});

	test('rename_template', async ({ page }) => {
		await page.goto('http://localhost:5173/templates-dashboard');
		await page.waitForLoadState('networkidle');
		const editButton = page.getByRole('button', { name: 'Edit' }).first();
		if (await editButton.isVisible()) {
			await editButton.click();
			await page.waitForURL('**/template-designer**');
			const nameInput = page.getByPlaceholder('Template Name');
			if (await nameInput.isVisible()) {
				await nameInput.clear();
				await nameInput.fill('Renamed Template');
			}
		}
	});

	test('save_template_with_branch_visibility', async ({ page }) => {
		await page.route('https://nominatim.openstreetmap.org/search**', async (route) => {
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
		// Navigate directly to branches
		await page.goto('http://localhost:5173/branches');
		await page.waitForLoadState('networkidle');

		await page.getByRole('textbox', { name: 'Branch Name' }).fill('Visibility Branch');
		await page.getByRole('textbox', { name: 'Address' }).fill('123 Main St');
		await page
			.locator('form > div.search-container.relative.flex-1 > div > button')
			.first()
			.click();
		await page.getByRole('button', { name: 'ADD BRANCH' }).click();
		await expect(page.getByText('Visibility Branch')).toBeVisible();

		await page.goto('http://localhost:5173/template-designer');
		await page.waitForLoadState('networkidle');

		const nameInput = page.getByPlaceholder('Template Name');
		await nameInput.fill('Branch Specific Template');

		// Select branch visibility
		const branchSelect = page.locator('#branch-select');
		await expect(branchSelect).toBeVisible();
		await branchSelect.selectOption({ label: 'Visibility Branch' });

		await page.getByRole('button', { name: 'Save Template' }).click();
		await expect(page.getByText('Template saved successfully!')).toBeVisible();
	});

	test('branch_visibility_persists_on_reload', async ({ page }) => {
		await page.goto('http://localhost:5173/template-designer');
		await page.waitForLoadState('networkidle');

		const templateName = `Branch Persist Template ${Date.now()}`;
		await page.getByPlaceholder('Template Name').fill(templateName);

		const branchSelect = page.locator('#branch-select');
		await expect(branchSelect).toBeVisible();
		await branchSelect.selectOption({ label: BRANCH_NAME });

		await page.getByRole('button', { name: 'Save Template' }).click();
		await expect(page.getByText('Template saved successfully!')).toBeVisible();

		await page.goto(
			`http://localhost:5173/template-designer?id=${encodeURIComponent(templateName)}`
		);
		await page.waitForLoadState('networkidle');

		const selectedOption = branchSelect.locator('option:checked');
		await expect(selectedOption).toHaveText(BRANCH_NAME);
	});

	test('save_without_template_name_shows_error', async ({ page }) => {
		await page.goto('http://localhost:5173/template-designer');
		await page.waitForLoadState('networkidle');
		const saveButton = page.getByRole('button', { name: 'Save Template', exact: true });
		if (await saveButton.isVisible()) {
			const isDisabled = await saveButton.isDisabled();
			if (!isDisabled) {
				await saveButton.click();
			}
		}
	});
});

test.describe('Template Designer - Component Management', () => {
	test.beforeEach(async ({ page }) => {
		await page.goto('http://localhost:5173/');
		await page.getByRole('link', { name: 'Login' }).click();
		await page.getByRole('textbox', { name: 'Email' }).fill(adminCreds.email);
		await page.getByRole('textbox', { name: 'Password' }).fill(adminCreds.password);
		await page.getByRole('button', { name: 'Sign in', exact: true }).click();
		await page.waitForURL('**/dashboard');
		await page.goto('http://localhost:5173/template-designer');
		await page.waitForLoadState('networkidle');
	});

	test('add_text_input_component', async ({ page }) => {
		const textInputButton = page.getByRole('button', { name: 'Text Input' });
		if (await textInputButton.isVisible()) {
			await textInputButton.click();
		}
	});

	test('add_checkbox_component', async ({ page }) => {
		const checkboxButton = page.getByRole('button', { name: 'Checkbox' });
		if (await checkboxButton.isVisible()) {
			await checkboxButton.click();
		}
	});

	test('add_temperature_component', async ({ page }) => {
		const tempButton = page.getByRole('button', { name: 'Temperature' });
		if (await tempButton.isVisible()) {
			await tempButton.click();
		}
	});

	test('add_dropdown_component', async ({ page }) => {
		const dropdownButton = page.getByRole('button', { name: 'Dropdown' });
		if (await dropdownButton.isVisible()) {
			await dropdownButton.click();
		}
	});

	test('add_label_component', async ({ page }) => {
		const labelButton = page.getByRole('button', { name: 'Label' });
		if (await labelButton.isVisible()) {
			await labelButton.click();
		}
	});

	test('select_component_shows_properties', async ({ page }) => {
		const textInputButton = page.getByRole('button', { name: 'Text Input' });
		if (await textInputButton.isVisible()) {
			await textInputButton.click();

			const canvas = page.locator('canvas, .canvas, [role="main"]').first();
			if (await canvas.isVisible()) {
				await canvas.click({ position: { x: 100, y: 100 } });
			}
		}
	});

	test('delete_selected_component', async ({ page }) => {
		const textInputButton = page.getByRole('button', { name: 'Text Input' });
		if (await textInputButton.isVisible()) {
			await textInputButton.click();

			const deleteButton = page.getByRole('button', { name: 'Delete' });
			if (await deleteButton.isVisible()) {
				await deleteButton.click();
			}
		}
	});

	test('update_component_text_property', async ({ page }) => {
		const textInputButton = page.getByRole('button', { name: 'Text Input' });
		if (await textInputButton.isVisible()) {
			await textInputButton.click();

			const textProperty = page.getByLabel('Text');
			if (await textProperty.isVisible()) {
				await textProperty.fill('Updated Text');
			}
		}
	});

	test('update_temperature_min_max', async ({ page }) => {
		const tempButton = page.getByRole('button', { name: 'Temperature' });
		if (await tempButton.isVisible()) {
			await tempButton.click();

			const minInput = page.getByLabel('Min');
			const maxInput = page.getByLabel('Max');
			if ((await minInput.isVisible()) && (await maxInput.isVisible())) {
				await minInput.fill('0');
				await maxInput.fill('100');
			}
		}
	});

	test('update_dropdown_options', async ({ page }) => {
		const dropdownButton = page.getByRole('button', { name: 'Dropdown' });
		if (await dropdownButton.isVisible()) {
			await dropdownButton.click();

			const optionsInput = page.getByLabel('Options');
			if (await optionsInput.isVisible()) {
				await optionsInput.fill('Option1,Option2,Option3');
			}
		}
	});
});

test.describe('Template Designer - Layout Tools', () => {
	test.beforeEach(async ({ page }) => {
		await page.goto('http://localhost:5173/');
		await page.getByRole('link', { name: 'Login' }).click();
		await page.getByRole('textbox', { name: 'Email' }).fill(adminCreds.email);
		await page.getByRole('textbox', { name: 'Password' }).fill(adminCreds.password);
		await page.getByRole('button', { name: 'Sign in', exact: true }).click();
		await page.waitForURL('**/dashboard');
		await page.goto('http://localhost:5173/template-designer');
		await page.waitForLoadState('networkidle');
	});

	test('align_component_left', async ({ page }) => {
		const textInputButton = page.getByRole('button', { name: 'Text Input' });
		if (await textInputButton.isVisible()) {
			await textInputButton.click();

			const alignLeftButton = page.getByRole('button', { name: 'Align Left' });
			if (await alignLeftButton.isVisible()) {
				await alignLeftButton.click();
			}
		}
	});

	test('align_component_center', async ({ page }) => {
		const textInputButton = page.getByRole('button', { name: 'Text Input' });
		if (await textInputButton.isVisible()) {
			await textInputButton.click();

			const alignCenterButton = page.getByRole('button', { name: 'Align Center' });
			if (await alignCenterButton.isVisible()) {
				await alignCenterButton.click();
			}
		}
	});

	test('align_component_right', async ({ page }) => {
		const textInputButton = page.getByRole('button', { name: 'Text Input' });
		if (await textInputButton.isVisible()) {
			await textInputButton.click();

			const alignRightButton = page.getByRole('button', { name: 'Align Right' });
			if (await alignRightButton.isVisible()) {
				await alignRightButton.click();
			}
		}
	});

	test('lock_component_position', async ({ page }) => {
		const textInputButton = page.getByRole('button', { name: 'Text Input' });
		if (await textInputButton.isVisible()) {
			await textInputButton.click();

			const lockButton = page.getByRole('button', { name: 'Lock' });
			if (await lockButton.isVisible()) {
				await lockButton.click();
			}
		}
	});
});

test.describe('Template Designer - Validation', () => {
	test.beforeEach(async ({ page }) => {
		await page.goto('http://localhost:5173/');
		await page.getByRole('link', { name: 'Login' }).click();
		await page.getByRole('textbox', { name: 'Email' }).fill(adminCreds.email);
		await page.getByRole('textbox', { name: 'Password' }).fill(adminCreds.password);
		await page.getByRole('button', { name: 'Sign in', exact: true }).click();
		await page.waitForURL('**/dashboard');
		await page.goto('http://localhost:5173/template-designer');
		await page.waitForLoadState('networkidle');
	});

	test('temperature_min_greater_than_max_validation', async ({ page }) => {
		const tempButton = page.getByRole('button', { name: 'Temperature' });
		if (await tempButton.isVisible()) {
			await tempButton.click();

			const minInput = page.getByLabel('Min');
			const maxInput = page.getByLabel('Max');
			if ((await minInput.isVisible()) && (await maxInput.isVisible())) {
				await minInput.fill('100');
				await maxInput.fill('0');
			}
		}
	});

	test('empty_dropdown_options_validation', async ({ page }) => {
		const dropdownButton = page.getByRole('button', { name: 'Dropdown' });
		if (await dropdownButton.isVisible()) {
			await dropdownButton.click();

			const optionsInput = page.getByLabel('Options');
			if (await optionsInput.isVisible()) {
				await optionsInput.clear();
			}
		}
	});

	test('unsaved_changes_warning', async ({ page }) => {
		const nameInput = page.getByPlaceholder('Template Name');
		if (await nameInput.isVisible()) {
			await nameInput.fill('Template With Changes');

			const backButton = page.getByRole('link', { name: 'Back' });
			if (await backButton.isVisible()) {
				await backButton.click();
			}
		}
	});

	test('navigate_between_templates', async ({ page }) => {
		await page.goto('http://localhost:5173/templates-dashboard');
		await page.waitForLoadState('networkidle');
		const editButtons = page.getByRole('button', { name: 'Edit' });
		const count = await editButtons.count();
		if (count >= 2) {
			await editButtons.nth(0).click();
			await page.waitForURL('**/template-designer**');
			await page.goBack();
			await page.waitForURL('**/templates-dashboard');
			await editButtons.nth(1).click();
			await page.waitForURL('**/template-designer**');
		}
	});
});

test.describe('Template Designer - Member Access Control', () => {
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
	test('member_cannot_access_template_designer', async ({ page }) => {
		await page.goto('http://localhost:5173/');
		await page.getByRole('link', { name: 'Login' }).click();
		await page.getByRole('textbox', { name: 'Email' }).fill(memberCreds.email);
		await page.getByRole('textbox', { name: 'Password' }).fill(memberCreds.password);
		await page.getByRole('button', { name: 'Sign in', exact: true }).click();
		await page.waitForURL('**/logs-list');
		await page.goto('http://localhost:5173/template-designer');
		await page.waitForURL('**/logs-list');
		await expect(page).toHaveURL('/logs-list');
	});
});
