import { expect, test } from '@playwright/test';
import { register } from './utils';

test.describe('Reports route behavior', () => {
	test('report run APIs reject unauthenticated users and work when authenticated', async ({
		page,
		browser
	}) => {
		const unauthCreate = await page.request.post('http://localhost:6767/reports/runs', {
			headers: { 'Content-Type': 'application/json' },
			data: {
				params: {
					date_from_iso: '2026-01-01',
					date_to_iso: '2026-01-01',
					selected_branch_ids: [],
					selected_log_type_ids: ['type1'],
					arrange_by: 'date',
					include_temperature_graphs: false,
					params_version: 1
				}
			}
		});
		if (unauthCreate.status() === 404) {
			test.skip(true, 'Backend test environment does not expose /reports/runs yet');
		}
		expect(unauthCreate.status()).toBe(401);

		const unauthList = await page.request.get('http://localhost:6767/reports/runs?limit=5');
		expect(unauthList.status()).toBe(401);

		const creds = await register(browser, false);
		if (!creds || !creds.page) {
			throw new Error('Failed to register authenticated user');
		}

		const token = (await creds.page.context().cookies()).find(
			(cookie) => cookie.name === 'ls-token'
		)?.value;
		if (!token) {
			throw new Error('Missing auth token cookie for authenticated route checks');
		}

		const authHeaders = {
			Authorization: `Bearer ${token}`,
			'Content-Type': 'application/json'
		};

		const listRes = await page.request.get('http://localhost:6767/reports/runs?limit=5', {
			headers: authHeaders
		});
		expect(listRes.status()).toBe(200);

		const isoDate = new Date().toISOString().slice(0, 10);
		const createRes = await page.request.post('http://localhost:6767/reports/runs', {
			headers: authHeaders,
			data: {
				name: 'Playwright Saved Report',
				params: {
					date_from_iso: isoDate,
					date_to_iso: isoDate,
					selected_branch_ids: [],
					selected_log_type_ids: ['type1', 'type2', 'type3', 'type4'],
					arrange_by: 'date',
					include_temperature_graphs: false,
					params_version: 1
				}
			}
		});
		expect(createRes.status()).toBe(201);

		const createJson = await createRes.json();
		const reportRunId = createJson?.report_run?.id as string | undefined;
		expect(reportRunId).toBeTruthy();

		const useRes = await page.request.post(
			`http://localhost:6767/reports/runs/${reportRunId}/use`,
			{
				headers: authHeaders
			}
		);
		expect(useRes.status()).toBe(200);

		const deleteRes = await page.request.delete(
			`http://localhost:6767/reports/runs/${reportRunId}`,
			{
				headers: authHeaders
			}
		);
		expect(deleteRes.status()).toBe(200);

		const postDeleteUse = await page.request.post(
			`http://localhost:6767/reports/runs/${reportRunId}/use`,
			{
				headers: authHeaders
			}
		);
		expect(postDeleteUse.status()).toBe(404);

		await creds.page.close();
	});

	test('log type filtering excludes temperature entries when temperature toggle is off', async ({
		browser
	}) => {
		const creds = await register(browser, false);
		if (!creds || !creds.page) {
			throw new Error('Failed to register user for reports filter test');
		}

		const page = creds.page;
		const token = (await page.context().cookies()).find(
			(cookie) => cookie.name === 'ls-token'
		)?.value;
		if (!token) {
			throw new Error('Missing auth token cookie for report filtering test');
		}

		const authHeaders = {
			Authorization: `Bearer ${token}`,
			'Content-Type': 'application/json'
		};

		const suffix = `${Date.now()}-${Math.floor(Math.random() * 100000)}`;
		const tempTemplate = `PW-Temp-${suffix}`;
		const textTemplate = `PW-Text-${suffix}`;

		const createTemplate = async (templateName: string, fieldType: 'temperature' | 'text') => {
			const response = await page.request.post('http://localhost:6767/logs/templates', {
				headers: authHeaders,
				data: {
					template_name: templateName,
					template_layout: [
						{
							field_type: fieldType,
							position: { x: 0, y: 0 },
							props:
								fieldType === 'temperature'
									? { text: 'Temperature', min: 0, max: 100, unit: 'C', required: true }
									: { text: 'Notes', required: true }
						}
					],
					schedule: {
						frequency: 'Daily',
						days_of_week: [1, 2, 3, 4, 5, 6, 7]
					}
				}
			});

			expect(response.ok()).toBeTruthy();
		};

		const createAndSubmitEntry = async (templateName: string, value: number | string) => {
			const createRes = await page.request.post('http://localhost:6767/logs/entries', {
				headers: authHeaders,
				data: { template_name: templateName }
			});
			expect(createRes.status()).toBe(201);

			const createJson = await createRes.json();
			const entryId = createJson?.id as string | undefined;
			expect(entryId).toBeTruthy();

			const updateRes = await page.request.put(`http://localhost:6767/logs/entries/${entryId}`, {
				headers: authHeaders,
				data: { entry_data: { 0: value } }
			});
			expect(updateRes.ok()).toBeTruthy();

			const submitRes = await page.request.post(
				`http://localhost:6767/logs/entries/${entryId}/submit`,
				{
					headers: authHeaders,
					data: {}
				}
			);
			expect(submitRes.ok()).toBeTruthy();
		};

		await createTemplate(tempTemplate, 'temperature');
		await createTemplate(textTemplate, 'text');
		await createAndSubmitEntry(tempTemplate, 22);
		await createAndSubmitEntry(textTemplate, 'All good');

		await expect
			.poll(
				async () => {
					const res = await page.request.get('http://localhost:6767/logs/admin/entries', {
						headers: authHeaders
					});
					if (!res.ok()) return 0;
					const json = (await res.json()) as { entries?: Array<{ template_name?: string }> };
					return (
						json.entries?.filter(
							(entry) =>
								entry.template_name === tempTemplate || entry.template_name === textTemplate
						).length || 0
					);
				},
				{ timeout: 15000 }
			)
			.toBeGreaterThanOrEqual(2);

		await page.goto('http://localhost:5173/reports');
		await page.waitForURL('**/reports');

		const logTypeFieldset = page.getByRole('group', { name: 'Log Types:' });
		const allTypeButton = logTypeFieldset.getByRole('button', { name: 'All', exact: true });
		const textTypeButton = logTypeFieldset.getByRole('button', { name: 'Text', exact: true });
		const tempTypeButton = logTypeFieldset.getByRole('button', {
			name: 'Temperature',
			exact: true
		});

		await tempTypeButton.click();
		await expect(allTypeButton).toHaveAttribute('aria-pressed', 'false');
		await expect(textTypeButton).toHaveAttribute('aria-pressed', 'true');
		await expect(tempTypeButton).toHaveAttribute('aria-pressed', 'false');

		await page.waitForTimeout(500);

		await page.getByRole('button', { name: 'Generate' }).click();

		await expect
			.poll(async () => (await page.locator('body').textContent()) || '', { timeout: 15000 })
			.toContain('Log Report');

		await expect(page.locator('body')).toContainText(textTemplate);

		await expect(page.locator('body')).not.toContainText(tempTemplate);

		await page.close();
	});
});

test.describe('Users admin save error handling', () => {
	test('save button is re-enabled after network failure on desktop sidebar', async ({
		browser
	}) => {
		const creds = await register(browser, false);
		if (!creds || !creds.page) {
			throw new Error('Failed to register user for save-flow test');
		}

		const page = creds.page;
		await page.setViewportSize({ width: 1400, height: 1000 });
		let saveAttempts = 0;

		await page.route('**/api/auth/admin/update-member', async (route) => {
			saveAttempts += 1;
			await route.abort('failed');
		});

		await page.goto('http://localhost:5173/users-admin');
		await page.waitForURL('**/users-admin');

		for (let i = 0; i < 5; i++) {
			await page.locator(`button:has-text("${creds.email}")`).first().click();
			if (await page.locator('#userSidebar').isVisible()) {
				break;
			}
			await page.waitForTimeout(300);
		}
		await expect(page.locator('#userSidebar')).toBeVisible();

		await page.locator('#fname').fill(`${creds.firstName}-retry`);

		const saveButton = page
			.locator('#userSidebar')
			.getByRole('button', { name: 'Save', exact: true });
		await saveButton.click();
		await expect(page.getByText('Failed to update member: network error')).toBeVisible();
		await expect(saveButton).toBeEnabled();

		await saveButton.click();
		expect(saveAttempts).toBeGreaterThanOrEqual(2);
		await expect(saveButton).toBeEnabled();

		await page.close();
	});
});
