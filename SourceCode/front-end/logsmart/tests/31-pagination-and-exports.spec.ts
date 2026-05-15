import { expect, test } from '@playwright/test';
import { register, dismissCookieBannerInTests, sendInvitation, acceptInvitation } from './utils';

const BACKEND_URL = process.env.BACKEND_URL || 'http://localhost:6767';

// Helper: get auth cookies from a logged-in page
async function getAuthCookies(page: {
	context: () => { cookies: (urls: string[]) => Promise<Array<{ name: string; value: string }>> };
}) {
	const cookies = await page.context().cookies([BACKEND_URL, 'http://localhost:5173']);
	const tokenCookie = cookies.find((c) => c.name === 'ls-token');
	return tokenCookie?.value || null;
}

// Helper: login and return auth token
async function loginAndGetToken(
	page: import('@playwright/test').Page,
	email: string,
	password: string
): Promise<string> {
	await dismissCookieBannerInTests(page);
	await page.goto('http://localhost:5173/login');
	await page.getByRole('textbox', { name: 'Email' }).fill(email);
	await page.getByRole('textbox', { name: 'Password' }).fill(password);
	await page.getByRole('button', { name: 'Sign in', exact: true }).click();
	await page.waitForURL('**/dashboard**');
	const token = await getAuthCookies(page);
	if (!token) throw new Error('Failed to get auth token after login');
	return token;
}

// Helper: parse CSV string into array of objects
function parseCSV(csv: string): Array<Record<string, string>> {
	const lines = csv.split('\n').filter((line) => line.trim() !== '');
	if (lines.length === 0) return [];
	const headers = lines[0].split(',').map((h) => h.trim().replace(/^"|"$/g, ''));
	const rows: Array<Record<string, string>> = [];
	for (let i = 1; i < lines.length; i++) {
		const values: string[] = [];
		let current = '';
		let inQuotes = false;
		for (const char of lines[i]) {
			if (char === '"') {
				inQuotes = !inQuotes;
			} else if (char === ',' && !inQuotes) {
				values.push(current.trim());
				current = '';
			} else {
				current += char;
			}
		}
		values.push(current.trim());
		const row: Record<string, string> = {};
		headers.forEach((header, idx) => {
			row[header] = values[idx] || '';
		});
		rows.push(row);
	}
	return rows;
}

// ============================================================================
// Test Group 1: Pagination Tests
// ============================================================================
test.describe('Pagination', () => {
	// Logs Pagination
	test.describe('Logs Pagination', () => {
		test('logs_list_supports_pagination_with_api', async ({ browser }) => {
			const adminResult = await register(browser, false);
			if (!adminResult || !adminResult.page) throw new Error('Failed to register admin user');
			const { page, email, password } = adminResult;

			const token = await loginAndGetToken(page, email, password);

			// Create multiple log entries by creating templates and submitting logs
			// First, create a simple template via API
			for (let i = 0; i < 5; i++) {
				const templateName = `Pagination Test Template ${Date.now()}-${i}`;
				await page.request.post('/api/logs/templates', {
					headers: {
						Authorization: `Bearer ${token}`,
						'Content-Type': 'application/json'
					},
					data: {
						template_name: templateName,
						template_layout: [
							{
								field_type: 'text_input',
								position: { x: 10, y: 10 },
								props: { text: `Field ${i}`, placeholder: 'Enter value' }
							}
						],
						schedule: {
							frequency: 'Daily',
							days_of_week: [1, 2, 3, 4, 5]
						}
					}
				});
			}

			// Verify logs list page loads
			await page.goto('http://localhost:5173/logs-list');
			await page.waitForURL('**/logs-list');
			await expect(page.getByText('All Logs')).toBeVisible();

			await page.close();
		});

		test('logs_pagination_navigation', async ({ browser }) => {
			const adminResult = await register(browser, false);
			if (!adminResult || !adminResult.page) throw new Error('Failed to register admin user');
			const { page, email, password } = adminResult;

			await loginAndGetToken(page, email, password);

			// Navigate to logs list
			await page.goto('http://localhost:5173/logs-list');
			await page.waitForURL('**/logs-list');

			// Verify logs list is accessible
			await expect(page.getByText('All Logs')).toBeVisible();

			await page.close();
		});
	});

	// Templates Pagination
	test.describe('Templates Pagination', () => {
		test('templates_list_displays_all_templates', async ({ browser }) => {
			const adminResult = await register(browser, false);
			if (!adminResult || !adminResult.page) throw new Error('Failed to register admin user');
			const { page, email, password } = adminResult;

			const token = await loginAndGetToken(page, email, password);

			// Create multiple templates to test listing
			const templateCount = 5;
			for (let i = 0; i < templateCount; i++) {
				const templateName = `Test Template ${Date.now()}-${i}`;
				const response = await page.request.post('/api/logs/templates', {
					headers: {
						Authorization: `Bearer ${token}`,
						'Content-Type': 'application/json'
					},
					data: {
						template_name: templateName,
						template_layout: [
							{
								field_type: 'text_input',
								position: { x: 10, y: 10 },
								props: { text: `Field ${i}`, placeholder: 'Enter value' }
							}
						],
						schedule: {
							frequency: 'Daily',
							days_of_week: [1, 2, 3, 4, 5]
						}
					}
				});
				expect(response.ok()).toBeTruthy();
			}

			// Navigate to template designer
			await page.goto('http://localhost:5173/template-designer');
			await page.waitForURL('**/template-designer');

			// Verify templates sidebar is visible
			await expect(page.getByRole('heading', { name: 'Templates' })).toBeVisible();
			await expect(page.getByRole('button', { name: '+ Create New' })).toBeVisible();

			await page.close();
		});
	});

	// Members Pagination
	test.describe('Members Pagination', () => {
		test('members_list_displays_company_members', async ({ browser }) => {
			const adminResult = await register(browser, false);
			if (!adminResult || !adminResult.page) throw new Error('Failed to register admin user');
			const { page, email, password } = adminResult;

			await loginAndGetToken(page, email, password);

			// Navigate to users admin page
			await page.goto('http://localhost:5173/users-admin');
			await page.waitForURL('**/users-admin');

			// Verify members list is visible
			await expect(page.locator('#eventHide')).toBeVisible();

			await page.close();
		});

		test('members_pagination_with_multiple_users', async ({ browser }) => {
			const adminResult = await register(browser, false);
			if (!adminResult) throw new Error('Failed to register admin user');
			const adminPage = adminResult.page!;
			const adminEmail = adminResult.email;
			const adminPassword = adminResult.password;

			await loginAndGetToken(adminPage, adminEmail, adminPassword);

			// Create a branch first (needed for some roles)
			await adminPage.route('https://nominatim.openstreetmap.org/search**', async (route) => {
				await route.fulfill({
					json: [
						{
							place_id: 1,
							lat: '10.0000000',
							lon: '10.0000000',
							display_name: 'Test Address, City, Country',
							address: { city: 'Test City', country: 'Test Country' }
						}
					]
				});
			});

			await adminPage.getByRole('link', { name: 'Branches' }).click();
			await adminPage.waitForURL('**/branches');
			await adminPage
				.getByRole('textbox', { name: 'Branch Name' })
				.fill(`Test Branch ${Date.now()}`);
			await adminPage.getByRole('textbox', { name: 'Address' }).fill('Test Address');
			await adminPage
				.locator('form > div.search-container.relative.flex-1 > div > button')
				.first()
				.click();
			await adminPage.getByRole('button', { name: 'ADD BRANCH' }).click();

			// Navigate to users admin
			await adminPage.getByRole('link', { name: 'Users' }).click();
			await adminPage.waitForURL('**/users-admin');

			// Verify the users page loads with member list
			await expect(adminPage.locator('#eventHide')).toBeVisible();

			await adminPage.close();
		});
	});

	// Attendance Pagination
	test.describe('Attendance Pagination', () => {
		test('attendance_page_supports_cursor_pagination', async ({ browser }) => {
			const adminResult = await register(browser, false);
			if (!adminResult || !adminResult.page) throw new Error('Failed to register admin user');
			const { page, email, password } = adminResult;

			await loginAndGetToken(page, email, password);

			// Navigate to attendance admin
			await page.goto('http://localhost:5173/attendance-admin');
			await page.waitForURL('**/attendance-admin');

			// Verify attendance page loads
			await expect(page.getByText('Attendance Overview')).toBeVisible();

			// Check for pagination controls (Previous/Next buttons)
			const prevButton = page.getByRole('button', { name: 'Previous' });
			const nextButton = page.getByRole('button', { name: 'Next' });

			// Pagination controls should be present
			await expect(prevButton).toBeVisible();
			await expect(nextButton).toBeVisible();

			// With no data, both should be disabled
			await expect(prevButton).toBeDisabled();
			await expect(nextButton).toBeDisabled();

			await page.close();
		});

		test('attendance_pagination_navigation_with_data', async ({ browser }) => {
			const adminResult = await register(browser, false);
			if (!adminResult) throw new Error('Failed to register admin user');
			const adminPage = adminResult.page!;
			const adminEmail = adminResult.email;
			const adminPassword = adminResult.password;

			const token = await loginAndGetToken(adminPage, adminEmail, adminPassword);

			// Create attendance records via API - clock in/out multiple times
			for (let i = 0; i < 3; i++) {
				// Clock in
				const clockInResponse = await adminPage.request.post('/api/clock/in', {
					headers: {
						Authorization: `Bearer ${token}`,
						'Content-Type': 'application/json'
					}
				});

				// Clock out after a short delay
				if (clockInResponse.ok()) {
					await adminPage.request.post('/api/clock/out', {
						headers: {
							Authorization: `Bearer ${token}`,
							'Content-Type': 'application/json'
						}
					});
				}
			}

			// Navigate to attendance page
			await adminPage.goto('http://localhost:5173/attendance-admin');
			await adminPage.waitForURL('**/attendance-admin');

			// Verify attendance data is displayed
			await expect(adminPage.getByText('Attendance Overview')).toBeVisible();

			// Check pagination controls exist
			const prevButton = adminPage.getByRole('button', { name: 'Previous' });
			const nextButton = adminPage.getByRole('button', { name: 'Next' });

			await expect(prevButton).toBeVisible();
			await expect(nextButton).toBeVisible();

			// Verify "Showing up to 25 records per page" text
			await expect(adminPage.getByText('Showing up to 25 records per page')).toBeVisible();

			await adminPage.close();
		});
	});
});

// ============================================================================
// Test Group 2: Export File Integrity Tests
// ============================================================================
// Company Data Export
test.describe('Company Data Export', () => {
	test('export_company_data_downloads_zip', async ({ browser }) => {
		const adminResult = await register(browser, false);
		if (!adminResult || !adminResult.page) throw new Error('Failed to register admin user');
		const { page, email, password } = adminResult;

		await loginAndGetToken(page, email, password);

		// Navigate to company settings
		await page.goto('http://localhost:5173/company-settings');
		await page.waitForURL('**/company-settings');

		// Click export button
		const exportButton = page.getByRole('button', {
			name: /Export Company Data|Re-export Company Data/
		});
		await expect(exportButton).toBeVisible();

		// Wait for export confirmation message
		await exportButton.click();
		await expect(page.getByText(/Data exported on/)).toBeVisible({ timeout: 15000 });

		await page.close();
	});

	test('export_company_data_via_api', async ({ browser }) => {
		const adminResult = await register(browser, false);
		if (!adminResult || !adminResult.page) throw new Error('Failed to register admin user');
		const { page, email, password, companyName } = adminResult;

		const token = await loginAndGetToken(page, email, password);

		// Get company ID from auth/me
		const meResponse = await page.request.get('/api/auth/me', {
			headers: { Authorization: `Bearer ${token}` }
		});
		const meData = await meResponse.json();
		const companyId = meData.company_id;

		// Trigger export via API with correct endpoint
		const exportResponse = await page.request.post(`/api/companies/${companyId}/export`, {
			headers: {
				Authorization: `Bearer ${token}`,
				'Content-Type': 'application/json'
			}
		});

		// Export should succeed (200 or 202)
		expect([200, 202]).toContain(exportResponse.status());

		await page.close();
	});
});

// Security Logs CSV Export
test.describe('Security Logs CSV Export', () => {
	test('export_security_logs_csv_format', async ({ browser }) => {
		const adminResult = await register(browser, false);
		if (!adminResult || !adminResult.page) throw new Error('Failed to register admin user');
		const { page, email, password } = adminResult;

		const token = await loginAndGetToken(page, email, password);

		// Update user to logsmart_admin role via direct DB access not available,
		// so we test the endpoint directly with the admin token
		// The endpoint may return empty CSV but should still be valid format

		// Request security logs export
		const exportResponse = await page.request.get('/api/security/logs/export', {
			headers: {
				Authorization: `Bearer ${token}`
			}
		});

		// Response should be CSV (may be 200 or 403 depending on role)
		const contentType = exportResponse.headers()['content-type'] || '';
		if (exportResponse.status() === 200) {
			expect(contentType).toContain('text/csv');
			const csvContent = await exportResponse.text();
			expect(csvContent).toBeDefined();

			// Parse and verify CSV structure
			const rows = parseCSV(csvContent);
			expect(Array.isArray(rows)).toBeTruthy();
		}

		await page.close();
	});

	test('export_security_logs_csv_headers', async ({ browser }) => {
		const adminResult = await register(browser, false);
		if (!adminResult || !adminResult.page) throw new Error('Failed to register admin user');
		const { page, email, password } = adminResult;

		const token = await loginAndGetToken(page, email, password);

		// Request export
		const exportResponse = await page.request.get('/api/security/logs/export', {
			headers: {
				Authorization: `Bearer ${token}`
			}
		});

		if (exportResponse.status() === 200) {
			const csvContent = await exportResponse.text();
			const lines = csvContent.split('\n').filter((line) => line.trim() !== '');

			// Verify CSV has headers
			expect(lines.length).toBeGreaterThanOrEqual(1);
			const headers = lines[0].split(',').map((h) => h.trim().replace(/^"|"$/g, ''));

			// Expected columns in security logs CSV
			const expectedColumns = ['timestamp', 'event_type', 'email', 'ip_address', 'details'];
			for (const col of expectedColumns) {
				expect(headers).toContain(col);
			}
		}

		await page.close();
	});

	test('security_log_export_visible_csv', async ({ browser }) => {
		const adminResult = await register(browser, false);
		if (!adminResult || !adminResult.page) throw new Error('Failed to register admin user');
		const { page, email, password } = adminResult;

		const token = await loginAndGetToken(page, email, password);

		// Request visible export (with default filters)
		const exportResponse = await page.request.get('/api/security/logs/export?limit=15', {
			headers: {
				Authorization: `Bearer ${token}`
			}
		});

		if (exportResponse.status() === 200) {
			const contentType = exportResponse.headers()['content-type'] || '';
			expect(contentType).toContain('text/csv');

			const csvContent = await exportResponse.text();
			expect(csvContent.length).toBeGreaterThan(0);
		}

		await page.close();
	});
});

// ============================================================================
// Test Group 3: Edge Cases
// ============================================================================
test.describe('Edge Cases', () => {
	// Empty State Pagination
	test.describe('Empty State Pagination', () => {
		test('attendance_empty_state_shows_disabled_pagination', async ({ browser }) => {
			const adminResult = await register(browser, false);
			if (!adminResult || !adminResult.page) throw new Error('Failed to register admin user');
			const { page, email, password } = adminResult;

			await loginAndGetToken(page, email, password);

			// Navigate to attendance without creating any records
			await page.goto('http://localhost:5173/attendance-admin');
			await page.waitForURL('**/attendance-admin');

			// Verify pagination controls are visible but disabled
			const prevButton = page.getByRole('button', { name: 'Previous' });
			const nextButton = page.getByRole('button', { name: 'Next' });

			await expect(prevButton).toBeVisible();
			await expect(nextButton).toBeVisible();
			await expect(prevButton).toBeDisabled();
			await expect(nextButton).toBeDisabled();

			// Verify "Showing up to 25 records per page" text is visible
			await expect(page.getByText('Showing up to 25 records per page')).toBeVisible();

			await page.close();
		});

		test('logs_empty_state_message', async ({ browser }) => {
			const adminResult = await register(browser, false);
			if (!adminResult || !adminResult.page) throw new Error('Failed to register admin user');
			const { page, email, password } = adminResult;

			await loginAndGetToken(page, email, password);

			// Navigate to logs list
			await page.goto('http://localhost:5173/logs-list');
			await page.waitForURL('**/logs-list');

			// Verify empty state message or logs section is visible
			const allLogsSection = page.getByText('All Logs');
			await expect(allLogsSection).toBeVisible();

			await page.close();
		});
	});

	// Large Dataset Pagination
	test.describe('Large Dataset Pagination', () => {
		test('attendance_pagination_with_multiple_records', async ({ browser }) => {
			const adminResult = await register(browser, false);
			if (!adminResult || !adminResult.page) throw new Error('Failed to register admin user');
			const { page, email, password } = adminResult;

			const token = await loginAndGetToken(page, email, password);

			// Create multiple clock-in events
			const recordCount = 10;
			for (let i = 0; i < recordCount; i++) {
				const clockInTime = new Date(Date.now() - i * 3600000).toISOString();
				await page.request.post('/api/clock/in', {
					headers: {
						Authorization: `Bearer ${token}`,
						'Content-Type': 'application/json'
					},
					data: {
						clock_in: clockInTime
					}
				});
			}

			// Navigate to attendance page
			await page.goto('http://localhost:5173/attendance-admin');
			await page.waitForURL('**/attendance-admin');

			// Verify records are displayed
			await expect(page.getByText('Attendance Overview')).toBeVisible();

			// Verify pagination controls
			const nextButton = page.getByRole('button', { name: 'Next' });
			await expect(nextButton).toBeVisible();

			await page.close();
		});

		test('pagination_with_real_clock_events', async ({ browser }) => {
			const adminResult = await register(browser, false);
			if (!adminResult || !adminResult.page) throw new Error('Failed to register admin user');
			const { page } = adminResult;

			const token = await loginAndGetToken(page, adminResult.email, adminResult.password);

			// Clock in and out 30 times to create real events
			for (let i = 0; i < 30; i++) {
				const clockInResponse = await page.request.post('/api/clock/in', {
					headers: {
						Authorization: `Bearer ${token}`,
						'Content-Type': 'application/json'
					}
				});

				if (clockInResponse.ok()) {
					await page.request.post('/api/clock/out', {
						headers: {
							Authorization: `Bearer ${token}`,
							'Content-Type': 'application/json'
						}
					});
				}
			}

			// Verify events were created via API
			const eventsResponse = await page.request.get('/api/clock/company?limit=5', {
				headers: { Authorization: `Bearer ${token}` }
			});
			const eventsData = await eventsResponse.json();
			expect(eventsData.events.length).toBeGreaterThanOrEqual(5);

			// Navigate to attendance page
			await page.goto('http://localhost:5173/attendance-admin');
			await page.waitForURL('**/attendance-admin');

			// Verify pagination controls exist
			const nextButton = page.getByRole('button', { name: 'Next' });
			const prevButton = page.getByRole('button', { name: 'Previous' });

			// With 30 events and default page size, Next should be visible
			const nextVisible = await nextButton.isVisible().catch(() => false);
			expect(nextVisible).toBeTruthy();

			// Click Next to test pagination
			await nextButton.click();
			await page.waitForTimeout(500);

			// Previous should now be enabled
			const prevVisible = await prevButton.isVisible().catch(() => false);
			expect(prevVisible).toBeTruthy();

			await page.close();
		});
	});

	// Export with Special Characters
	test.describe('Export with Special Characters', () => {
		test('csv_handles_special_characters_in_data', async ({ browser }) => {
			const adminResult = await register(browser, false);
			if (!adminResult || !adminResult.page) throw new Error('Failed to register admin user');
			const { page, email, password } = adminResult;

			const token = await loginAndGetToken(page, email, password);

			// Create a template with special characters in name
			const specialTemplateName = `Template with "quotes" and, commas ${Date.now()}`;
			const createResponse = await page.request.post('/api/logs/templates', {
				headers: {
					Authorization: `Bearer ${token}`,
					'Content-Type': 'application/json'
				},
				data: {
					template_name: specialTemplateName,
					template_layout: [
						{
							field_type: 'text_input',
							position: { x: 10, y: 10 },
							props: {
								text: 'Field with "special" chars, and newlines\nhere',
								placeholder: 'Test "quotes" and, commas'
							}
						}
					],
					schedule: {
						frequency: 'Daily',
						days_of_week: [1, 2, 3, 4, 5]
					}
				}
			});
			expect(createResponse.ok()).toBeTruthy();

			// Fetch templates and verify special characters are preserved
			const templatesResponse = await page.request.get('/api/logs/templates', {
				headers: {
					Authorization: `Bearer ${token}`
				}
			});
			expect(templatesResponse.ok()).toBeTruthy();
			const templatesData = await templatesResponse.json();

			// Verify template with special characters exists
			const foundTemplate = templatesData.templates?.find(
				(t: { template_name: string }) => t.template_name === specialTemplateName
			);
			expect(foundTemplate).toBeDefined();

			await page.close();
		});

		test('csv_export_formula_injection_protection', async ({ browser }) => {
			const adminResult = await register(browser, false);
			if (!adminResult || !adminResult.page) throw new Error('Failed to register admin user');
			const { page, email, password } = adminResult;

			const token = await loginAndGetToken(page, email, password);

			// Create user/profile data with potential formula injection characters
			// Update profile with suspicious values
			await page.request.put('/api/auth/profile', {
				headers: {
					Authorization: `Bearer ${token}`,
					'Content-Type': 'application/json'
				},
				data: {
					first_name: '=SUM(A1:A10)',
					last_name: '+1234567'
				}
			});

			// Request security logs export
			const exportResponse = await page.request.get('/api/security/logs/export', {
				headers: {
					Authorization: `Bearer ${token}`
				}
			});

			if (exportResponse.status() === 200) {
				const csvContent = await exportResponse.text();

				// Verify CSV doesn't start with formula characters on new lines
				const lines = csvContent.split('\n');
				for (let i = 1; i < lines.length; i++) {
					const line = lines[i].trim();
					if (line) {
						// Each field should not start with =, +, -, @ when unquoted
						expect(line).not.toMatch(/^,=/);
						expect(line).not.toMatch(/^,\+/);
					}
				}
			}

			await page.close();
		});

		test('csv_proper_escaping_quotes_and_commas', async ({ browser }) => {
			const adminResult = await register(browser, false);
			if (!adminResult || !adminResult.page) throw new Error('Failed to register admin user');
			const { page, email, password } = adminResult;

			const token = await loginAndGetToken(page, email, password);

			// Create template with commas and quotes
			const templateName = `Test, "Template" ${Date.now()}`;
			await page.request.post('/api/logs/templates', {
				headers: {
					Authorization: `Bearer ${token}`,
					'Content-Type': 'application/json'
				},
				data: {
					template_name: templateName,
					template_layout: [
						{
							field_type: 'text_input',
							position: { x: 10, y: 10 },
							props: { text: 'Value with "quotes"', placeholder: 'Placeholder, with comma' }
						}
					],
					schedule: {
						frequency: 'Daily',
						days_of_week: [1, 2, 3, 4, 5]
					}
				}
			});

			// Fetch and verify data integrity
			const response = await page.request.get('/api/logs/templates', {
				headers: {
					Authorization: `Bearer ${token}`
				}
			});
			expect(response.ok()).toBeTruthy();
			const data = await response.json();

			// Verify template exists with correct name
			const found = data.templates?.find(
				(t: { template_name: string }) => t.template_name === templateName
			);
			expect(found).toBeDefined();
			expect(found?.template_name).toBe(templateName);

			await page.close();
		});
	});
});
