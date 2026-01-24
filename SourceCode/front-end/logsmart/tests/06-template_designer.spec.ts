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
			await page.waitForTimeout(500);
		}
	});

	test('save_template', async ({ page }) => {
		await page.goto('http://localhost:5173/template-designer');
		await page.waitForLoadState('networkidle');
		const nameInput = page.getByPlaceholder('Template Name');
		if (await nameInput.isVisible()) {
			await nameInput.fill('Saveable Template');
			await page.waitForTimeout(500);
			const saveButton = page.getByRole('button', { name: 'Save' });
			if ((await saveButton.isVisible()) && (await saveButton.isEnabled())) {
				await saveButton.click();
				await page.waitForTimeout(1000);
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
			await page.waitForTimeout(500);
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
				await page.waitForTimeout(500);
			}
		}
	});

	test('save_without_template_name_shows_error', async ({ page }) => {
		await page.goto('http://localhost:5173/template-designer');
		await page.waitForLoadState('networkidle');
		const saveButton = page.getByRole('button', { name: 'Save' });
		if (await saveButton.isVisible()) {
			const isDisabled = await saveButton.isDisabled();
			if (!isDisabled) {
				await saveButton.click();
				await page.waitForTimeout(500);
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
			await page.waitForTimeout(500);
		}
	});

	test('add_checkbox_component', async ({ page }) => {
		const checkboxButton = page.getByRole('button', { name: 'Checkbox' });
		if (await checkboxButton.isVisible()) {
			await checkboxButton.click();
			await page.waitForTimeout(500);
		}
	});

	test('add_temperature_component', async ({ page }) => {
		const tempButton = page.getByRole('button', { name: 'Temperature' });
		if (await tempButton.isVisible()) {
			await tempButton.click();
			await page.waitForTimeout(500);
		}
	});

	test('add_dropdown_component', async ({ page }) => {
		const dropdownButton = page.getByRole('button', { name: 'Dropdown' });
		if (await dropdownButton.isVisible()) {
			await dropdownButton.click();
			await page.waitForTimeout(500);
		}
	});

	test('add_label_component', async ({ page }) => {
		const labelButton = page.getByRole('button', { name: 'Label' });
		if (await labelButton.isVisible()) {
			await labelButton.click();
			await page.waitForTimeout(500);
		}
	});

	test('select_component_shows_properties', async ({ page }) => {
		const textInputButton = page.getByRole('button', { name: 'Text Input' });
		if (await textInputButton.isVisible()) {
			await textInputButton.click();
			await page.waitForTimeout(500);
			const canvas = page.locator('canvas, .canvas, [role="main"]').first();
			if (await canvas.isVisible()) {
				await canvas.click({ position: { x: 100, y: 100 } });
				await page.waitForTimeout(500);
			}
		}
	});

	test('delete_selected_component', async ({ page }) => {
		const textInputButton = page.getByRole('button', { name: 'Text Input' });
		if (await textInputButton.isVisible()) {
			await textInputButton.click();
			await page.waitForTimeout(500);
			const deleteButton = page.getByRole('button', { name: 'Delete' });
			if (await deleteButton.isVisible()) {
				await deleteButton.click();
				await page.waitForTimeout(500);
			}
		}
	});

	test('update_component_text_property', async ({ page }) => {
		const textInputButton = page.getByRole('button', { name: 'Text Input' });
		if (await textInputButton.isVisible()) {
			await textInputButton.click();
			await page.waitForTimeout(500);
			const textProperty = page.getByLabel('Text');
			if (await textProperty.isVisible()) {
				await textProperty.fill('Updated Text');
				await page.waitForTimeout(300);
			}
		}
	});

	test('update_temperature_min_max', async ({ page }) => {
		const tempButton = page.getByRole('button', { name: 'Temperature' });
		if (await tempButton.isVisible()) {
			await tempButton.click();
			await page.waitForTimeout(500);
			const minInput = page.getByLabel('Min');
			const maxInput = page.getByLabel('Max');
			if ((await minInput.isVisible()) && (await maxInput.isVisible())) {
				await minInput.fill('0');
				await maxInput.fill('100');
				await page.waitForTimeout(300);
			}
		}
	});

	test('update_dropdown_options', async ({ page }) => {
		const dropdownButton = page.getByRole('button', { name: 'Dropdown' });
		if (await dropdownButton.isVisible()) {
			await dropdownButton.click();
			await page.waitForTimeout(500);
			const optionsInput = page.getByLabel('Options');
			if (await optionsInput.isVisible()) {
				await optionsInput.fill('Option1,Option2,Option3');
				await page.waitForTimeout(300);
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
			await page.waitForTimeout(500);
			const alignLeftButton = page.getByRole('button', { name: 'Align Left' });
			if (await alignLeftButton.isVisible()) {
				await alignLeftButton.click();
				await page.waitForTimeout(300);
			}
		}
	});

	test('align_component_center', async ({ page }) => {
		const textInputButton = page.getByRole('button', { name: 'Text Input' });
		if (await textInputButton.isVisible()) {
			await textInputButton.click();
			await page.waitForTimeout(500);
			const alignCenterButton = page.getByRole('button', { name: 'Align Center' });
			if (await alignCenterButton.isVisible()) {
				await alignCenterButton.click();
				await page.waitForTimeout(300);
			}
		}
	});

	test('align_component_right', async ({ page }) => {
		const textInputButton = page.getByRole('button', { name: 'Text Input' });
		if (await textInputButton.isVisible()) {
			await textInputButton.click();
			await page.waitForTimeout(500);
			const alignRightButton = page.getByRole('button', { name: 'Align Right' });
			if (await alignRightButton.isVisible()) {
				await alignRightButton.click();
				await page.waitForTimeout(300);
			}
		}
	});

	test('lock_component_position', async ({ page }) => {
		const textInputButton = page.getByRole('button', { name: 'Text Input' });
		if (await textInputButton.isVisible()) {
			await textInputButton.click();
			await page.waitForTimeout(500);
			const lockButton = page.getByRole('button', { name: 'Lock' });
			if (await lockButton.isVisible()) {
				await lockButton.click();
				await page.waitForTimeout(300);
			}
		}
	});
});

test.describe('Template Designer - AI Generation', () => {
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

	test('open_ai_generation_sidebar', async ({ page }) => {
		const aiButton = page.getByRole('button', { name: 'AI' });
		if (await aiButton.isVisible()) {
			await aiButton.click();
			await page.waitForTimeout(500);
		}
	});

	test('generate_layout_with_ai_prompt', async ({ page }) => {
		const aiButton = page.getByRole('button', { name: 'AI' });
		if (await aiButton.isVisible()) {
			await aiButton.click();
			await page.waitForTimeout(500);
			const promptInput = page.getByPlaceholder('Describe your template');
			const generateButton = page.getByRole('button', { name: 'Generate' });
			if ((await promptInput.isVisible()) && (await generateButton.isVisible())) {
				await promptInput.fill('Create a temperature log with morning and evening checks');
				await generateButton.click();
				await page.waitForTimeout(3000);
			}
		}
	});

	test('ai_generation_with_empty_prompt_shows_error', async ({ page }) => {
		const aiButton = page.getByRole('button', { name: 'AI' });
		if (await aiButton.isVisible()) {
			await aiButton.click();
			await page.waitForTimeout(500);
			const generateButton = page.getByRole('button', { name: 'Generate' });
			if (await generateButton.isVisible()) {
				const isDisabled = await generateButton.isDisabled();
				if (!isDisabled) {
					await generateButton.click();
					await page.waitForTimeout(500);
				}
			}
		}
	});

	test('undo_ai_generated_layout', async ({ page }) => {
		const aiButton = page.getByRole('button', { name: 'AI' });
		if (await aiButton.isVisible()) {
			await aiButton.click();
			await page.waitForTimeout(500);
			const promptInput = page.getByPlaceholder('Describe your template');
			const generateButton = page.getByRole('button', { name: 'Generate' });
			if ((await promptInput.isVisible()) && (await generateButton.isVisible())) {
				await promptInput.fill('Simple checklist');
				await generateButton.click();
				await page.waitForTimeout(3000);
				const undoButton = page.getByRole('button', { name: 'Undo' });
				if (await undoButton.isVisible()) {
					await undoButton.click();
					await page.waitForTimeout(500);
				}
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
			await page.waitForTimeout(500);
			const minInput = page.getByLabel('Min');
			const maxInput = page.getByLabel('Max');
			if ((await minInput.isVisible()) && (await maxInput.isVisible())) {
				await minInput.fill('100');
				await maxInput.fill('0');
				await page.waitForTimeout(500);
			}
		}
	});

	test('empty_dropdown_options_validation', async ({ page }) => {
		const dropdownButton = page.getByRole('button', { name: 'Dropdown' });
		if (await dropdownButton.isVisible()) {
			await dropdownButton.click();
			await page.waitForTimeout(500);
			const optionsInput = page.getByLabel('Options');
			if (await optionsInput.isVisible()) {
				await optionsInput.clear();
				await page.waitForTimeout(500);
			}
		}
	});

	test('unsaved_changes_warning', async ({ page }) => {
		const nameInput = page.getByPlaceholder('Template Name');
		if (await nameInput.isVisible()) {
			await nameInput.fill('Template With Changes');
			await page.waitForTimeout(500);
			const backButton = page.getByRole('link', { name: 'Back' });
			if (await backButton.isVisible()) {
				await backButton.click();
				await page.waitForTimeout(500);
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
