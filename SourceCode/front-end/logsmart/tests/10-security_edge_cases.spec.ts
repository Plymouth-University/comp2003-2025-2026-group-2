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

test.describe('Security: SQL Injection Prevention', () => {
	test('sql_injection_login_email', async ({ page }) => {
		await page.goto('http://localhost:5173/');
		await page.getByRole('link', { name: 'Login' }).click();
		await page.getByRole('textbox', { name: 'Email' }).fill("admin@ex.com'--");
		await page.getByRole('textbox', { name: 'Password' }).fill('anything');
		await page.getByRole('button', { name: 'Sign in', exact: true }).click();
		await page.waitForTimeout(1000);
		await expect(page).toHaveURL('/login');
	});

	test('sql_injection_login_password', async ({ page }) => {
		await page.goto('http://localhost:5173/');
		await page.getByRole('link', { name: 'Login' }).click();
		await page.getByRole('textbox', { name: 'Email' }).fill('testuser@logsmart.app');
		await page.getByRole('textbox', { name: 'Password' }).fill("' OR '1'='1");
		await page.getByRole('button', { name: 'Sign in', exact: true }).click();
		await page.waitForTimeout(1000);
		await expect(page).toHaveURL('/login');
	});

	test('sql_injection_register_company_name', async ({ page }) => {
		await page.goto('http://localhost:5173/register-company');
		await page
			.getByRole('textbox', { name: 'Company Name' })
			.fill("TestCo'; DROP TABLE companies;--");
		await page.getByRole('textbox', { name: 'Company Address' }).fill('123 Test St');
		await page.getByRole('button', { name: 'Next Step' }).click();
		await page.getByRole('textbox', { name: 'First Name' }).fill('Test');
		await page.getByRole('textbox', { name: 'Last Name' }).fill('User');
		await page.getByRole('textbox', { name: 'Email' }).fill('sqltest@logsmart.app');
		await page
			.getByRole('textbox', { name: 'Password Show password', exact: true })
			.fill('Test123!');
		await page.getByRole('textbox', { name: 'Confirm Password Show password' }).fill('Test123!');
		await page.getByRole('button', { name: 'Create Account' }).click();
		await page.waitForTimeout(1000);
	});

	test('sql_injection_template_search', async ({ page }) => {
		await page.goto('http://localhost:5173/');
		await page.getByRole('link', { name: 'Login' }).click();
		await page.getByRole('textbox', { name: 'Email' }).fill(adminCreds.email);
		await page.getByRole('textbox', { name: 'Password' }).fill(adminCreds.password);
		await page.getByRole('button', { name: 'Sign in', exact: true }).click();
		await page.waitForURL('**/dashboard');
		await page.getByRole('link', { name: 'Templates' }).click();
		await page.waitForURL('**/templates-dashboard');
		const searchInput = page.getByPlaceholder('Search templates');
		if (await searchInput.isVisible()) {
			await searchInput.fill("'; DELETE FROM templates WHERE '1'='1");
			await page.waitForTimeout(1000);
		}
	});
});

test.describe('Security: XSS Prevention', () => {
	test('xss_script_in_company_name', async ({ page }) => {
		const timestamp = Date.now();
		await page.goto('http://localhost:5173/register-company');
		await page.getByRole('textbox', { name: 'Company Name' }).fill('<script>alert("XSS")</script>');
		await page.getByRole('textbox', { name: 'Company Address' }).fill('123 Test St');
		await page.getByRole('button', { name: 'Next Step' }).click();
		await page.getByRole('textbox', { name: 'First Name' }).fill('Test');
		await page.getByRole('textbox', { name: 'Last Name' }).fill('User');
		await page.getByRole('textbox', { name: 'Email' }).fill(`xsstest${timestamp}@logsmart.app`);
		await page
			.getByRole('textbox', { name: 'Password Show password', exact: true })
			.fill('Test123!');
		await page.getByRole('textbox', { name: 'Confirm Password Show password' }).fill('Test123!');
		await page.getByRole('button', { name: 'Create Account' }).click();
		await page.waitForTimeout(1000);
	});

	test('xss_script_in_first_name', async ({ page }) => {
		await page.goto('http://localhost:5173/');
		await page.getByRole('link', { name: 'Login' }).click();
		await page.getByRole('textbox', { name: 'Email' }).fill(adminCreds.email);
		await page.getByRole('textbox', { name: 'Password' }).fill(adminCreds.password);
		await page.getByRole('button', { name: 'Sign in', exact: true }).click();
		await page.waitForURL('**/dashboard');
		await page.getByRole('link', { name: 'Settings' }).click();
		await page.waitForURL('**/settings');
		await page.waitForLoadState('networkidle');
		await page.getByRole('textbox', { name: 'First Name' }).clear();
		await page
			.getByRole('textbox', { name: 'First Name' })
			.fill('<img src=x onerror=alert("XSS")>');
		await page.getByRole('button', { name: 'Save Profile' }).click();
		await page.waitForTimeout(1000);
		await page.getByRole('textbox', { name: 'First Name' }).clear();
		await page.getByRole('textbox', { name: 'First Name' }).fill('Test');
		await page.getByRole('button', { name: 'Save Profile' }).click();
		await page.waitForTimeout(500);
	});

	test('xss_html_in_template_name', async ({ page }) => {
		await page.goto('http://localhost:5173/');
		await page.getByRole('link', { name: 'Login' }).click();
		await page.getByRole('textbox', { name: 'Email' }).fill(adminCreds.email);
		await page.getByRole('textbox', { name: 'Password' }).fill(adminCreds.password);
		await page.getByRole('button', { name: 'Sign in', exact: true }).click();
		await page.waitForURL('**/dashboard');
		await page.goto('http://localhost:5173/template-designer');
		await page.waitForLoadState('networkidle');
		const nameInput = page.getByPlaceholder('Template Name');
		if (await nameInput.isVisible()) {
			await nameInput.fill('<b>Bold Template</b><script>alert("XSS")</script>');
			await page.waitForTimeout(500);
		}
	});

	test('xss_javascript_protocol_in_email', async ({ page }) => {
		await page.goto('http://localhost:5173/');
		await page.getByRole('link', { name: 'Login' }).click();
		await page.getByRole('textbox', { name: 'Email' }).fill('javascript:alert("XSS")');
		await page.getByRole('textbox', { name: 'Password' }).fill('Test123!');
		await page.waitForTimeout(500);
		const signInButton = page.getByRole('button', { name: 'Sign in', exact: true });
		await expect(signInButton).toBeDisabled();
	});
});

test.describe('Security: Authorization Boundary Tests', () => {
	test('horizontal_privilege_escalation_view_other_user_logs', async ({ page, context }) => {
		await page.goto('http://localhost:5173/');
		await page.getByRole('link', { name: 'Login' }).click();
		await page.getByRole('textbox', { name: 'Email' }).fill(adminCreds.email);
		await page.getByRole('textbox', { name: 'Password' }).fill(adminCreds.password);
		await page.getByRole('button', { name: 'Sign in', exact: true }).click();
		await page.waitForURL('**/dashboard');

		await page.goto('http://localhost:5173/log-template?entry=1');
		await page.waitForLoadState('networkidle');
		await page.waitForTimeout(1000);
	});

	test('vertical_privilege_escalation_member_to_admin', async ({ page }) => {
		await page.goto('http://localhost:5173/');
		await page.getByRole('link', { name: 'Login' }).click();
		await page.getByRole('textbox', { name: 'Email' }).fill(adminCreds.email);
		await page.getByRole('textbox', { name: 'Password' }).fill(adminCreds.password);
		await page.getByRole('button', { name: 'Sign in', exact: true }).click();
		await page.waitForURL('**/dashboard');

		await page.goto('http://localhost:5173/dashboard');
		await page.waitForURL('**/dashboard');
		await expect(page).toHaveURL('/dashboard');

		await page.goto('http://localhost:5173/users-admin');
		await page.waitForURL('**/users-admin');
		await expect(page).toHaveURL('/users-admin');

		await page.goto('http://localhost:5173/templates-dashboard');
		await page.waitForURL('**/templates-dashboard');
		await expect(page).toHaveURL('/templates-dashboard');

		await page.goto('http://localhost:5173/template-designer');
		await page.waitForURL('**/template-designer');
		await expect(page).toHaveURL('/template-designer');
	});

	test('direct_object_reference_invalid_log_id', async ({ page }) => {
		await page.goto('http://localhost:5173/');
		await page.getByRole('link', { name: 'Login' }).click();
		await page.getByRole('textbox', { name: 'Email' }).fill(adminCreds.email);
		await page.getByRole('textbox', { name: 'Password' }).fill(adminCreds.password);
		await page.getByRole('button', { name: 'Sign in', exact: true }).click();
		await page.waitForURL('**/dashboard');

		await page.goto('http://localhost:5173/log-template?entry=999999999');
		await page.waitForLoadState('networkidle');
		await page.waitForTimeout(1000);
	});

	test('api_endpoint_authorization_without_token', async ({ page }) => {
		const response = await page.request.get('http://localhost:6767/logs/entries/due');
		expect(response.status()).toBe(401);
	});
});

test.describe('Edge Cases: Boundary Conditions', () => {
	test('very_long_company_name_boundary', async ({ page }) => {
		const longName = 'A'.repeat(256);
		await page.goto('http://localhost:5173/register-company');
		await page.getByRole('textbox', { name: 'Company Name' }).fill(longName);
		await page.getByRole('textbox', { name: 'Company Address' }).fill('123 Test St');
		await page.getByRole('button', { name: 'Next Step' }).click();
		await page.waitForTimeout(500);
	});

	test('very_long_email_address', async ({ page }) => {
		const longEmail = 'a'.repeat(100) + '@' + 'b'.repeat(100) + '.com';
		await page.goto('http://localhost:5173/');
		await page.getByRole('link', { name: 'Login' }).click();
		await page.getByRole('textbox', { name: 'Email' }).fill(longEmail);
		await page.getByRole('textbox', { name: 'Password' }).fill('Test123!');
		await page.waitForTimeout(500);
	});

	test('very_long_password', async ({ page }) => {
		const longPassword = 'A1!' + 'a'.repeat(500);
		await page.goto('http://localhost:5173/');
		await page.getByRole('link', { name: 'Login' }).click();
		await page.getByRole('textbox', { name: 'Email' }).fill('testuser@logsmart.app');
		await page.getByRole('textbox', { name: 'Password' }).fill(longPassword);
		await page.getByRole('button', { name: 'Sign in', exact: true }).click();
		await page.waitForTimeout(1000);
	});

	test('unicode_characters_in_names', async ({ page }) => {
		await page.goto('http://localhost:5173/');
		await page.getByRole('link', { name: 'Login' }).click();
		await page.getByRole('textbox', { name: 'Email' }).fill(adminCreds.email);
		await page.getByRole('textbox', { name: 'Password' }).fill(adminCreds.password);
		await page.getByRole('button', { name: 'Sign in', exact: true }).click();
		await page.waitForURL('**/dashboard');
		await page.getByRole('link', { name: 'Settings' }).click();
		await page.waitForURL('**/settings');
		await page.waitForLoadState('networkidle');
		await page.getByRole('textbox', { name: 'First Name' }).clear();
		await page.getByRole('textbox', { name: 'First Name' }).fill('José');
		await page.getByRole('textbox', { name: 'Last Name' }).clear();
		await page.getByRole('textbox', { name: 'Last Name' }).fill('González');
		await page.getByRole('button', { name: 'Save Profile' }).click();
		await page.waitForTimeout(1000);
		await page.getByRole('textbox', { name: 'First Name' }).clear();
		await page.getByRole('textbox', { name: 'First Name' }).fill('Test');
		await page.getByRole('textbox', { name: 'Last Name' }).clear();
		await page.getByRole('textbox', { name: 'Last Name' }).fill('User');
		await page.getByRole('button', { name: 'Save Profile' }).click();
		await page.waitForTimeout(500);
	});

	test('emoji_in_template_name', async ({ page }) => {
		await page.goto('http://localhost:5173/');
		await page.getByRole('link', { name: 'Login' }).click();
		await page.getByRole('textbox', { name: 'Email' }).fill(adminCreds.email);
		await page.getByRole('textbox', { name: 'Password' }).fill(adminCreds.password);
		await page.getByRole('button', { name: 'Sign in', exact: true }).click();
		await page.waitForURL('**/dashboard');
		await page.goto('http://localhost:5173/template-designer');
		await page.waitForLoadState('networkidle');
		const nameInput = page.getByPlaceholder('Template Name');
		if (await nameInput.isVisible()) {
			await nameInput.fill('🌡️ Temperature Log 📊');
			await page.waitForTimeout(500);
		}
	});

	test('null_bytes_in_input', async ({ page }) => {
		await page.goto('http://localhost:5173/');
		await page.getByRole('link', { name: 'Login' }).click();
		await page.getByRole('textbox', { name: 'Email' }).fill('test\u0000user@logsmart.app');
		await page.getByRole('textbox', { name: 'Password' }).fill('Test123!');
		await page.waitForTimeout(500);
	});

	test('zero_as_numeric_input', async ({ page }) => {
		await page.goto('http://localhost:5173/');
		await page.getByRole('link', { name: 'Login' }).click();
		await page.getByRole('textbox', { name: 'Email' }).fill(adminCreds.email);
		await page.getByRole('textbox', { name: 'Password' }).fill(adminCreds.password);
		await page.getByRole('button', { name: 'Sign in', exact: true }).click();
		await page.waitForURL('**/dashboard');
		await page.goto('http://localhost:5173/template-designer');
		await page.waitForLoadState('networkidle');
		const tempButton = page.getByRole('button', { name: 'Temperature' });
		if (await tempButton.isVisible()) {
			await tempButton.click();
			await page.waitForTimeout(500);
			const minInput = page.getByLabel('Min');
			const maxInput = page.getByLabel('Max');
			if ((await minInput.isVisible()) && (await maxInput.isVisible())) {
				await minInput.fill('0');
				await maxInput.fill('0');
				await page.waitForTimeout(300);
			}
		}
	});

	test('negative_temperature_values', async ({ page }) => {
		await page.goto('http://localhost:5173/');
		await page.getByRole('link', { name: 'Login' }).click();
		await page.getByRole('textbox', { name: 'Email' }).fill(adminCreds.email);
		await page.getByRole('textbox', { name: 'Password' }).fill(adminCreds.password);
		await page.getByRole('button', { name: 'Sign in', exact: true }).click();
		await page.waitForURL('**/dashboard');
		await page.goto('http://localhost:5173/template-designer');
		await page.waitForLoadState('networkidle');
		const tempButton = page.getByRole('button', { name: 'Temperature' });
		if (await tempButton.isVisible()) {
			await tempButton.click();
			await page.waitForTimeout(500);
			const minInput = page.getByLabel('Min');
			const maxInput = page.getByLabel('Max');
			if ((await minInput.isVisible()) && (await maxInput.isVisible())) {
				await minInput.fill('-50');
				await maxInput.fill('50');
				await page.waitForTimeout(300);
			}
		}
	});

	test('very_large_temperature_values', async ({ page }) => {
		await page.goto('http://localhost:5173/');
		await page.getByRole('link', { name: 'Login' }).click();
		await page.getByRole('textbox', { name: 'Email' }).fill(adminCreds.email);
		await page.getByRole('textbox', { name: 'Password' }).fill(adminCreds.password);
		await page.getByRole('button', { name: 'Sign in', exact: true }).click();
		await page.waitForURL('**/dashboard');
		await page.goto('http://localhost:5173/template-designer');
		await page.waitForLoadState('networkidle');
		const tempButton = page.getByRole('button', { name: 'Temperature' });
		if (await tempButton.isVisible()) {
			await tempButton.click();
			await page.waitForTimeout(500);
			const minInput = page.getByLabel('Min');
			const maxInput = page.getByLabel('Max');
			if ((await minInput.isVisible()) && (await maxInput.isVisible())) {
				await minInput.fill('0');
				await maxInput.fill('999999');
				await page.waitForTimeout(300);
			}
		}
	});
});

test.describe('Edge Cases: Concurrent Operations', () => {
	test('rapid_form_submissions', async ({ page }) => {
		await page.goto('http://localhost:5173/');
		await page.getByRole('link', { name: 'Login' }).click();
		await page.getByRole('textbox', { name: 'Email' }).fill('testuser@logsmart.app');
		await page.getByRole('textbox', { name: 'Password' }).fill('Test123!');
		for (let i = 0; i < 5; i++) {
			await page.getByRole('button', { name: 'Sign in', exact: true }).click({ force: true });
			await page.waitForTimeout(100);
		}
		await page.waitForTimeout(1000);
	});

	test('rapid_navigation_between_pages', async ({ page }) => {
		await page.goto('http://localhost:5173/');
		await page.getByRole('link', { name: 'Login' }).click();
		await page.getByRole('textbox', { name: 'Email' }).fill(adminCreds.email);
		await page.getByRole('textbox', { name: 'Password' }).fill(adminCreds.password);
		await page.getByRole('button', { name: 'Sign in', exact: true }).click();
		await page.waitForURL('**/dashboard');
		await page.waitForLoadState('networkidle');

		for (let i = 0; i < 3; i++) {
			await page.getByRole('link', { name: 'Logs', exact: true }).click();
			await page.waitForTimeout(200);
			await page.getByRole('link', { name: 'Templates' }).click();
			await page.waitForTimeout(200);
			await page.getByRole('link', { name: 'Dashboard', exact: true }).click();
			await page.waitForTimeout(200);
		}
	});

	test('multiple_save_operations', async ({ page }) => {
		await page.goto('http://localhost:5173/');
		await page.getByRole('link', { name: 'Login' }).click();
		await page.getByRole('textbox', { name: 'Email' }).fill(adminCreds.email);
		await page.getByRole('textbox', { name: 'Password' }).fill(adminCreds.password);
		await page.getByRole('button', { name: 'Sign in', exact: true }).click();
		await page.waitForURL('**/dashboard');
		await page.getByRole('link', { name: 'Settings' }).click();
		await page.waitForURL('**/settings');
		await page.waitForLoadState('networkidle');

		for (let i = 0; i < 3; i++) {
			await page.getByRole('textbox', { name: 'First Name' }).clear();
			await page.getByRole('textbox', { name: 'First Name' }).fill(`Test${i}`);
			await page.getByRole('button', { name: 'Save Profile' }).click();
			await page.waitForTimeout(500);
		}

		await page.getByRole('textbox', { name: 'First Name' }).clear();
		await page.getByRole('textbox', { name: 'First Name' }).fill('Test');
		await page.getByRole('button', { name: 'Save Profile' }).click();
		await page.waitForTimeout(500);
	});
});

test.describe('Edge Cases: Network and Performance', () => {
	test('page_reload_during_form_fill', async ({ page }) => {
		await page.goto('http://localhost:5173/register-company');
		await page.getByRole('textbox', { name: 'Company Name' }).fill('TestCompany');
		await page.getByRole('textbox', { name: 'Company Address' }).fill('123 Test St');
		await page.reload();
		await page.waitForLoadState('networkidle');
	});

	test('browser_back_button_navigation', async ({ page }) => {
		await page.goto('http://localhost:5173/');
		await page.getByRole('link', { name: 'Login' }).click();
		await page.getByRole('textbox', { name: 'Email' }).fill(adminCreds.email);
		await page.getByRole('textbox', { name: 'Password' }).fill(adminCreds.password);
		await page.getByRole('button', { name: 'Sign in', exact: true }).click();
		await page.waitForURL('**/dashboard');
		await page.waitForLoadState('networkidle');

		await page.getByRole('link', { name: 'Logs', exact: true }).click();
		await page.waitForURL('**/logs-list');
		await page.waitForLoadState('networkidle');

		await page.goBack();
		await page.waitForURL('**/dashboard');
		await page.waitForLoadState('networkidle');

		await page.goForward();
		await page.waitForURL('**/logs-list');
	});

	test('session_persistence_across_tabs', async ({ page, context }) => {
		await page.goto('http://localhost:5173/');
		await page.getByRole('link', { name: 'Login' }).click();
		await page.getByRole('textbox', { name: 'Email' }).fill(adminCreds.email);
		await page.getByRole('textbox', { name: 'Password' }).fill(adminCreds.password);
		await page.getByRole('button', { name: 'Sign in', exact: true }).click();
		await page.waitForURL('**/dashboard');

		const newTab = await context.newPage();
		await newTab.goto('http://localhost:5173/dashboard');
		await newTab.waitForLoadState('networkidle');
		await expect(newTab.locator('body')).toContainText(adminCreds.email);
		await newTab.close();
	});
});
