import { test, expect } from '@playwright/test';
import { register } from './utils';

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

test.describe('Template Versioning', () => {
	test.beforeEach(async ({ page }) => {
		await page.goto('http://localhost:5173/');
		await page.getByRole('link', { name: 'Login' }).click();
		await page.getByRole('textbox', { name: 'Email' }).fill(adminCreds.email);
		await page.getByRole('textbox', { name: 'Password' }).fill(adminCreds.password);
		await page.getByRole('button', { name: 'Sign in', exact: true }).click();
		await page.waitForURL('**/dashboard');
	});

	test('create_versions_and_restore', async ({ page }) => {
		const templateName = `Versioning Test ${Date.now()}`;

		// 1. Go to designer
		await page.goto('http://localhost:5173/template-designer');
		await page.waitForLoadState('networkidle');

		// 2. Create initial template (Version 1)
		// Note: The placeholder in DesignCanvas is "Enter template title..."
		await page.getByPlaceholder('Enter template title...').fill(templateName);
		
		await page.getByRole('button', { name: 'Text Input' }).click();
		
		// The properties panel should appear. Verify the component was added.
		// Click on the canvas item to ensure selection if needed, though adding usually selects it.
		// await page.locator('.canvas-item').first().click();

		await page.getByLabel('Text').fill('Field V1');
		await page.getByRole('button', { name: 'Save' }).click();		await expect(page.getByText('Template saved successfully!')).toBeVisible();
		await page.waitForTimeout(1000); // Wait for save to complete

		// 3. Update template (Version 2)
		await page.getByLabel('Text').fill('Field V2');
		await page.getByRole('button', { name: 'Save' }).click();
		await expect(page.getByText('Template saved successfully!')).toBeVisible();
		await page.waitForTimeout(1000);

		// 4. Update template again (Version 3)
		await page.getByLabel('Text').fill('Field V3');
		await page.getByRole('button', { name: 'Save' }).click();
		await expect(page.getByText('Template saved successfully!')).toBeVisible();
		await page.waitForTimeout(1000);

		// 5. Open History
		await page.getByRole('button', { name: 'History' }).click();
		await expect(page.getByRole('dialog')).toBeVisible();
		await expect(page.getByText('Version History')).toBeVisible();

		// Should see previous versions (V1 and V2)
		const restoreButtons = page.getByRole('button', { name: 'Restore' });
		// Wait for buttons to appear
		await expect(restoreButtons.first()).toBeVisible();
		await expect(restoreButtons).toHaveCount(2);

		// 6. Restore Version 1
		// Setup dialog handler BEFORE clicking
		page.once('dialog', async (dialog) => {
			await dialog.accept();
		});

		// Restore the oldest version (last in list) which had "Field V1"
		await restoreButtons.last().click();

		// 7. Verify restoration
		// Modal should close
		await expect(page.getByRole('dialog')).not.toBeVisible();

		// Canvas should show V1 content
		// We need to click the component to see its properties
		// Assuming the canvas items are reset, the first item should correspond to the text input we added
		const canvas = page.locator('canvas, .canvas, [role="main"]').first();

		// Wait for reload (loadTemplate is async)
		await page.waitForTimeout(1000);

		// Click the component (assuming it's roughly where we placed it or default pos)
		// Default x,y is 0,0 usually or defined in addComponent.
		// We added it without moving it, so it's at default spawn location.
		// Let's try clicking the first item in the canvas DOM if possible, or coordinates.
		// The canvas renders items as divs usually? Let's check DesignCanvas implementation.
		// It renders <CanvasItemComponent>.

		const canvasItem = page.locator('.canvas-item').first();
		await canvasItem.click();

		// Check properties panel
		const textProperty = page.getByLabel('Text');
		await expect(textProperty).toHaveValue('Field V1');
	});
});
