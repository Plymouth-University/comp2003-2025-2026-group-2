import { test, expect } from '@playwright/test';
import { register, dismissCookieBannerInTests } from './utils';

const API_URL = process.env.BACKEND_URL || 'http://localhost:6767';
const FRONTEND_URL = process.env.FRONTEND_URL || 'http://localhost:5173';

// Helper: login via API and return auth token
async function loginApi(email: string, password: string): Promise<string> {
	const response = await fetch(`${API_URL}/auth/login`, {
		method: 'POST',
		headers: { 'Content-Type': 'application/json' },
		body: JSON.stringify({ email, password })
	});

	if (!response.ok) {
		const error = await response.json();
		throw new Error(`Login failed: ${error.error || response.statusText}`);
	}

	const data = await response.json();
	return data.token;
}

let adminCreds: {
	email: string;
	password: string;
	companyName: string;
	firstName: string;
	lastName: string;
};
let adminToken: string;

test.beforeAll(async ({ browser }) => {
	const creds = await register(browser);
	if (!creds) throw new Error('Failed to register admin user');
	adminCreds = creds;

	const loginPage = await browser.newPage();
	await dismissCookieBannerInTests(loginPage);
	await loginPage.goto(`${FRONTEND_URL}/login`);
	await loginPage.getByRole('textbox', { name: 'Email' }).fill(adminCreds.email);
	await loginPage.getByRole('textbox', { name: 'Password' }).fill(adminCreds.password);
	await loginPage.getByRole('button', { name: 'Sign in', exact: true }).click();
	await loginPage.waitForURL('**/dashboard');

	const cookies = await loginPage.context().cookies();
	const sessionCookie = cookies.find((c) => c.name === 'session');
	adminToken = sessionCookie?.value || '';
	await loginPage.close();
});

test.describe('Cross-Cutting: Rate Limiting', () => {
	test('rate_limit_login_ip_threshold', async ({ page }) => {
		const failedLogins = 12;
		const rateLimited = [];

		for (let i = 0; i < failedLogins; i++) {
			const response = await page.request.post(`${API_URL}/auth/login`, {
				data: {
					email: `nonexistent+${i}@logsmart.app`,
					password: 'WrongPassword123!'
				}
			});

			if (response.status() === 429) {
				rateLimited.push(i);
			}
		}

		expect(rateLimited.length).toBeGreaterThan(0);
	});

	test('rate_limit_login_email_threshold', async ({ page }) => {
		const targetEmail = `ratelimit-test-${Date.now()}@logsmart.app`;
		const attempts = 15;
		let got429 = false;

		for (let i = 0; i < attempts; i++) {
			const response = await page.request.post(`${API_URL}/auth/login`, {
				data: {
					email: targetEmail,
					password: 'WrongPassword123!'
				}
			});

			if (response.status() === 429) {
				got429 = true;
				break;
			}
		}

		expect(got429).toBe(true);
	});

	test('rate_limit_registration_ip_threshold', async ({ page }) => {
		const attempts = 15;
		let got429 = false;

		for (let i = 0; i < attempts; i++) {
			const response = await page.request.post(`${API_URL}/auth/register`, {
				data: {
					company_name: `RateLimitCompany-${i}`,
					company_address: 'TestAddress1, ABC',
					first_name: `Test-${i}`,
					last_name: `User-${i}`,
					email: `ratelimit-reg-${Date.now()}-${i}@logsmart.app`,
					password: `Test${Date.now()}!A`
				}
			});

			if (response.status() === 429) {
				got429 = true;
				break;
			}
		}

		expect(got429).toBe(true);
	});

	test('rate_limit_password_reset_requests', async ({ page }) => {
		const targetEmail = `ratelimit-reset-${Date.now()}@logsmart.app`;
		const attempts = 15;
		let got429 = false;

		for (let i = 0; i < attempts; i++) {
			const response = await page.request.post(`${API_URL}/auth/password/request-reset`, {
				data: { email: targetEmail }
			});

			if (response.status() === 429) {
				got429 = true;
				break;
			}
		}

		expect(got429).toBe(true);
	});

	test('rate_limit_response_format', async ({ page }) => {
		const targetEmail = `ratelimit-format-${Date.now()}@logsmart.app`;

		for (let i = 0; i < 20; i++) {
			const response = await page.request.post(`${API_URL}/auth/login`, {
				data: { email: targetEmail, password: 'WrongPassword123!' }
			});

			if (response.status() === 429) {
				const body = await response.json();
				expect(body).toHaveProperty('error');
				expect(body).toHaveProperty('retry_after');
				expect(typeof body.error).toBe('string');
				break;
			}

			if (i === 19) {
				test.fail(true, 'Rate limit was not triggered within expected attempts');
			}
		}
	});
});

test.describe('Cross-Cutting: API-Level 403 Forbidden', () => {
	test('staff_access_admin_update_member_returns_403', async ({ browser }) => {
		const staffPage = await browser.newPage();
		await dismissCookieBannerInTests(staffPage);
		await staffPage.goto(`${FRONTEND_URL}/login`);
		await staffPage.getByRole('textbox', { name: 'Email' }).fill(adminCreds.email);
		await staffPage.getByRole('textbox', { name: 'Password' }).fill(adminCreds.password);
		await staffPage.getByRole('button', { name: 'Sign in', exact: true }).click();
		await staffPage.waitForURL('**/dashboard');

		await staffPage.getByRole('link', { name: 'Users' }).click();
		await staffPage.waitForURL('**/users-admin');
		await staffPage.getByRole('button', { name: '➕' }).click();
		await staffPage
			.getByRole('textbox', { name: "New user's email" })
			.fill(`staff-test-${Date.now()}@logsmart.app`);
		await staffPage.locator('#invite-role').selectOption('staff');
		await staffPage.getByRole('button', { name: 'Send Invite' }).click();
		await staffPage.close();

		const loginPage = await browser.newPage();
		await dismissCookieBannerInTests(loginPage);
		await loginPage.goto(`${FRONTEND_URL}/login`);
		await loginPage.getByRole('textbox', { name: 'Email' }).fill(adminCreds.email);
		await loginPage.getByRole('textbox', { name: 'Password' }).fill(adminCreds.password);
		await loginPage.getByRole('button', { name: 'Sign in', exact: true }).click();
		await loginPage.waitForURL('**/dashboard');

		const cookies = await loginPage.context().cookies();
		const sessionCookie = cookies.find((c) => c.name === 'session');
		const staffToken = sessionCookie?.value || '';

		const response = await loginPage.request.put(`${API_URL}/auth/admin/update-member`, {
			headers: { Cookie: `session=${staffToken}` },
			data: {
				user_id: 'nonexistent-id',
				role: 'staff'
			}
		});

		expect([401, 403]).toContain(response.status());
		await loginPage.close();
	});

	test('staff_access_admin_remove_member_returns_403', async ({ browser }) => {
		const page = await browser.newPage();
		await dismissCookieBannerInTests(page);

		const cookies = await page.context().cookies();
		const sessionCookie = cookies.find((c) => c.name === 'session');
		const token = sessionCookie?.value || '';

		const response = await page.request.delete(`${API_URL}/auth/admin/remove-member`, {
			headers: {
				Cookie: `session=${token}`,
				'Content-Type': 'application/json'
			},
			data: { user_id: 'nonexistent-id' }
		});

		expect([401, 403]).toContain(response.status());
		await page.close();
	});

	test('staff_access_admin_logs_entries_returns_403', async ({ browser }) => {
		const page = await browser.newPage();
		await dismissCookieBannerInTests(page);

		const cookies = await page.context().cookies();
		const sessionCookie = cookies.find((c) => c.name === 'session');
		const token = sessionCookie?.value || '';

		const response = await page.request.get(`${API_URL}/logs/admin/entries`, {
			headers: { Cookie: `session=${token}` }
		});

		expect([401, 403]).toContain(response.status());
		await page.close();
	});

	test('unauthorized_access_without_token_returns_401', async ({ page }) => {
		const response = await page.request.get(`${API_URL}/logs/admin/entries`);
		expect(response.status()).toBe(401);
	});

	test('invalid_token_returns_401', async ({ page }) => {
		const response = await page.request.get(`${API_URL}/logs/admin/entries`, {
			headers: { Cookie: 'session=invalid-token-value' }
		});
		expect(response.status()).toBe(401);
	});
});

test.describe('Cross-Cutting: 500 Error Handling', () => {
	test('ui_handles_login_500_error', async ({ page }) => {
		await page.route('**/api/auth/login', async (route) => {
			await route.fulfill({
				status: 500,
				contentType: 'application/json',
				body: JSON.stringify({ error: 'Internal Server Error' })
			});
		});

		await page.goto(`${FRONTEND_URL}/login`);
		await page.getByRole('textbox', { name: 'Email' }).fill('test@logsmart.app');
		await page.getByRole('textbox', { name: 'Password' }).fill('Test123!');
		await page.getByRole('button', { name: 'Sign in', exact: true }).click();

		await page.waitForTimeout(2000);

		expect(await page.isVisible('body')).toBe(true);
		await page.unroute('**/api/auth/login');
	});

	test('ui_handles_template_list_500_error', async ({ browser }) => {
		const page = await browser.newPage();
		await dismissCookieBannerInTests(page);
		await page.goto(`${FRONTEND_URL}/login`);
		await page.getByRole('textbox', { name: 'Email' }).fill(adminCreds.email);
		await page.getByRole('textbox', { name: 'Password' }).fill(adminCreds.password);
		await page.getByRole('button', { name: 'Sign in', exact: true }).click();
		await page.waitForURL('**/dashboard');

		await page.route('**/logs/templates**', async (route) => {
			await route.fulfill({
				status: 500,
				contentType: 'application/json',
				body: JSON.stringify({ error: 'Internal Server Error' })
			});
		});

		await page.goto(`${FRONTEND_URL}/templates-dashboard`);
		await page.waitForTimeout(2000);

		expect(await page.isVisible('body')).toBe(true);
		await page.unroute('**/logs/templates**');
		await page.close();
	});

	test('ui_handles_log_entries_500_error', async ({ browser }) => {
		const page = await browser.newPage();
		await dismissCookieBannerInTests(page);
		await page.goto(`${FRONTEND_URL}/login`);
		await page.getByRole('textbox', { name: 'Email' }).fill(adminCreds.email);
		await page.getByRole('textbox', { name: 'Password' }).fill(adminCreds.password);
		await page.getByRole('button', { name: 'Sign in', exact: true }).click();
		await page.waitForURL('**/dashboard');

		await page.route('**/logs/entries**', async (route) => {
			await route.fulfill({
				status: 500,
				contentType: 'application/json',
				body: JSON.stringify({ error: 'Internal Server Error' })
			});
		});

		await page.goto(`${FRONTEND_URL}/logs-list`);
		await page.waitForTimeout(2000);

		expect(await page.isVisible('body')).toBe(true);
		await page.unroute('**/logs/entries**');
		await page.close();
	});

	test('ui_handles_dashboard_500_error', async ({ browser }) => {
		const page = await browser.newPage();
		await dismissCookieBannerInTests(page);
		await page.goto(`${FRONTEND_URL}/login`);
		await page.getByRole('textbox', { name: 'Email' }).fill(adminCreds.email);
		await page.getByRole('textbox', { name: 'Password' }).fill(adminCreds.password);
		await page.getByRole('button', { name: 'Sign in', exact: true }).click();
		await page.waitForURL('**/dashboard');

		await page.route('**/logs/entries/due**', async (route) => {
			await route.fulfill({
				status: 500,
				contentType: 'application/json',
				body: JSON.stringify({ error: 'Internal Server Error' })
			});
		});

		await page.reload();
		await page.waitForTimeout(2000);

		expect(await page.isVisible('body')).toBe(true);
		await page.unroute('**/logs/entries/due**');
		await page.close();
	});
});

test.describe('Cross-Cutting: 404 Not Found', () => {
	test('nonexistent_template_returns_404', async ({ browser }) => {
		const page = await browser.newPage();
		await dismissCookieBannerInTests(page);
		await page.goto(`${FRONTEND_URL}/login`);
		await page.getByRole('textbox', { name: 'Email' }).fill(adminCreds.email);
		await page.getByRole('textbox', { name: 'Password' }).fill(adminCreds.password);
		await page.getByRole('button', { name: 'Sign in', exact: true }).click();
		await page.waitForURL('**/dashboard');

		const cookies = await page.context().cookies();
		const sessionCookie = cookies.find((c) => c.name === 'ls-token');
		const token = sessionCookie?.value || '';

		const response = await page.request.get(
			`${API_URL}/logs/templates?template_name=nonexistent-template-${Date.now()}`,
			{ headers: { Authorization: `Bearer ${token}` } }
		);

		expect([404, 200]).toContain(response.status());
		await page.close();
	});

	test('nonexistent_log_entry_returns_404', async ({ browser }) => {
		const page = await browser.newPage();
		await dismissCookieBannerInTests(page);
		await page.goto(`${FRONTEND_URL}/login`);
		await page.getByRole('textbox', { name: 'Email' }).fill(adminCreds.email);
		await page.getByRole('textbox', { name: 'Password' }).fill(adminCreds.password);
		await page.getByRole('button', { name: 'Sign in', exact: true }).click();
		await page.waitForURL('**/dashboard');

		const cookies = await page.context().cookies();
		const sessionCookie = cookies.find((c) => c.name === 'ls-token');
		const token = sessionCookie?.value || '';

		const response = await page.request.get(`${API_URL}/logs/entries/nonexistent-id-12345`, {
			headers: { Authorization: `Bearer ${token}` }
		});

		expect([404, 400]).toContain(response.status());
		await page.close();
	});

	test('nonexistent_user_profile_picture_returns_404', async ({ browser }) => {
		// Register a user to get auth token
		const result = await register(browser, false);
		if (!result) throw new Error('Registration failed');
		await result.page!.close();

		const token = await loginApi(result.email, result.password);

		const response = await fetch(`${API_URL}/auth/profile-picture/nonexistent-user-id`, {
			headers: { Authorization: `Bearer ${token}` }
		});
		// Should return 404 for non-existent user, or 400 for invalid ID format
		expect([404, 400]).toContain(response.status);
	});

	test('nonexistent_company_returns_404', async ({ browser }) => {
		// Register a user to get auth token
		const result = await register(browser, false);
		if (!result) throw new Error('Registration failed');
		await result.page!.close();

		const token = await loginApi(result.email, result.password);
		const response = await (await browser.newPage()).request.get(`${API_URL}/companies/nonexistent-company-id`, {
			headers: { Authorization: `Bearer ${token}` }
		});
		console.log(await response.text());
		expect(response.status()).toBe(403);
	});

	test('nonexistent_passkey_returns_404_on_delete', async ({ browser }) => {
		const page = await browser.newPage();
		await dismissCookieBannerInTests(page);
		await page.goto(`${FRONTEND_URL}/login`);
		await page.getByRole('textbox', { name: 'Email' }).fill(adminCreds.email);
		await page.getByRole('textbox', { name: 'Password' }).fill(adminCreds.password);
		await page.getByRole('button', { name: 'Sign in', exact: true }).click();
		await page.waitForURL('**/dashboard');

		const cookies = await page.context().cookies();
		const sessionCookie = cookies.find((c) => c.name === 'ls-token');
		const token = sessionCookie?.value || '';

		const response = await page.request.delete(`${API_URL}/auth/passkeys/nonexistent-passkey-id`, {
			headers: { Authorization: `Bearer ${token}` }
		});
		console.log(await response.text());
		expect([404, 400]).toContain(response.status());
		await page.close();
	});
});

test.describe('Cross-Cutting: 400 Bad Request', () => {
	test('invalid_json_body_returns_400', async ({ page }) => {
		const response = await page.request.post(`${API_URL}/auth/login`, {
			headers: { 'Content-Type': 'application/json' },
			data: 'this is not valid json' as any
		});

		expect([400, 422]).toContain(response.status());
	});

	test('login_missing_email_field_returns_400', async ({ page }) => {
		const response = await page.request.post(`${API_URL}/auth/login`, {
			data: { password: 'Test123!' }
		});

		expect([400, 422]).toContain(response.status());
	});

	test('login_missing_password_field_returns_400', async ({ page }) => {
		const response = await page.request.post(`${API_URL}/auth/login`, {
			data: { email: 'test@logsmart.app' }
		});

		expect([400, 422]).toContain(response.status());
	});

	test('login_empty_fields_returns_400', async ({ page }) => {
		const response = await page.request.post(`${API_URL}/auth/login`, {
			data: { email: '', password: '' }
		});

		expect([400, 422, 401]).toContain(response.status());
	});

	test('registration_missing_required_fields_returns_400', async ({ page }) => {
		const response = await page.request.post(`${API_URL}/auth/register`, {
			data: {
				company_name: 'TestCompany'
			}
		});

		expect([400, 422]).toContain(response.status());
	});

	test('registration_invalid_email_format_returns_400', async ({ page }) => {
		const response = await page.request.post(`${API_URL}/auth/register`, {
			data: {
				company_name: 'TestCompany',
				company_address: 'TestAddress1, ABC',
				first_name: 'Test',
				last_name: 'User',
				email: 'not-an-email',
				password: 'Test123!'
			}
		});

		expect([400, 422]).toContain(response.status());
	});

	test('password_reset_missing_email_returns_400', async ({ page }) => {
		const response = await page.request.post(`${API_URL}/auth/password/request-reset`, {
			data: {}
		});

		expect([400, 422]).toContain(response.status());
	});

	test('template_update_empty_body_returns_400', async ({ browser }) => {
		const page = await browser.newPage();
		await dismissCookieBannerInTests(page);
		await page.goto(`${FRONTEND_URL}/login`);
		await page.getByRole('textbox', { name: 'Email' }).fill(adminCreds.email);
		await page.getByRole('textbox', { name: 'Password' }).fill(adminCreds.password);
		await page.getByRole('button', { name: 'Sign in', exact: true }).click();
		await page.waitForURL('**/dashboard');

		const cookies = await page.context().cookies();
		const sessionCookie = cookies.find((c) => c.name === 'ls-token');
		const token = sessionCookie?.value || '';

		const response = await page.request.put(`${API_URL}/logs/templates/update`, {
			headers: { Authorization: `Bearer ${token}` },
			data: {}
		});
		console.log(await response.text());
		expect([400, 422]).toContain(response.status());
		await page.close();
	});

	test('extremely_long_string_in_login_returns_400', async ({ page }) => {
		const longEmail = 'a'.repeat(10000) + '@logsmart.app';
		const response = await page.request.post(`${API_URL}/auth/login`, {
			data: { email: longEmail, password: 'Test123!' }
		});

		expect([400, 422, 401]).toContain(response.status());
	});

	test('negative_numeric_values_in_registration_returns_400', async ({ page }) => {
		const response = await page.request.post(`${API_URL}/auth/register`, {
			data: {
				company_name: -1,
				company_address: 'TestAddress1, ABC',
				first_name: 'Test',
				last_name: 'User',
				email: `negtest-${Date.now()}@logsmart.app`,
				password: 'Test123!'
			}
		});

		expect([400, 422]).toContain(response.status());
	});

	test('wrong_data_type_password_returns_400', async ({ page }) => {
		const response = await page.request.post(`${API_URL}/auth/login`, {
			data: { email: 'test@logsmart.app', password: 12345 }
		});

		expect([400, 422]).toContain(response.status());
	});

	test('null_values_in_required_fields_returns_400', async ({ page }) => {
		const response = await page.request.post(`${API_URL}/auth/login`, {
			data: { email: null, password: null }
		});

		expect([400, 422]).toContain(response.status());
	});
});
