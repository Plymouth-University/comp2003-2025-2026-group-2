import { expect, test } from '@playwright/test';
import { Client } from 'pg';
import { register } from './utils';

test.describe('Internal Admin Security Log', () => {
	test('security log tab renders and supports filters, pagination, and exports', async ({
		page,
		browser
	}) => {
		const creds = await register(browser);
		if (!creds) {
			throw new Error('Failed to register test user');
		}

		const databaseUrl =
			process.env.TEST_DATABASE_URL || 'postgres://admin:adminpassword@localhost:5432/logsmartdb';
		const client = new Client({ connectionString: databaseUrl });
		await client.connect();
		const result = await client.query('UPDATE users SET role = $1::user_role WHERE email = $2', [
			'logsmart_admin',
			creds.email
		]);
		if (result.rowCount === 0) {
			throw new Error('Failed to update user role for test user');
		}
		await client.end();

		await page.goto('http://localhost:5173/login');
		await page.getByRole('textbox', { name: 'Email' }).fill(creds.email);
		await page.getByRole('textbox', { name: 'Password' }).fill(creds.password);
		await page.getByRole('button', { name: 'Sign in', exact: true }).click();
		await page.waitForURL('**/dashboard');

		const tokenCookie = (await page.context().cookies()).find(
			(cookie) => cookie.name === 'ls-token'
		);
		if (!tokenCookie?.value) {
			throw new Error('Missing auth token cookie after login');
		}

		const profileResponse = await page.request.put('http://localhost:5173/api/auth/profile', {
			headers: {
				Authorization: `Bearer ${tokenCookie.value}`,
				'Content-Type': 'application/json'
			},
			data: {
				first_name: `${creds.firstName}-admin`,
				last_name: `${creds.lastName}-admin`
			}
		});
		expect(profileResponse.ok()).toBeTruthy();

		await page.goto('http://localhost:5173/admin-dashboard');
		await page.waitForURL('**/admin-dashboard');
		await page.getByRole('button', { name: 'Security Log' }).click();

		await expect(page.getByText('Showing up to 15 events per page')).toBeVisible();
		await expect(page.getByRole('button', { name: 'Apply Filters' })).toBeVisible();
		await expect(page.getByRole('button', { name: 'Export Visible CSV' })).toBeVisible();
		await expect(page.getByRole('button', { name: 'Export All CSV' })).toBeVisible();

		await page.locator('#security-filter-event-type').fill('login');
		await page.locator('#security-filter-email').fill('logsmart');
		await page.getByRole('button', { name: 'Apply Filters' }).click();

		await expect(page.getByRole('button', { name: 'Previous' })).toBeVisible();
		await expect(page.getByRole('button', { name: 'Next' })).toBeVisible();

		const visibleButton = page.getByRole('button', { name: 'Export Visible CSV' });
		if (await visibleButton.isEnabled()) {
			const [visibleDownload] = await Promise.all([
				page.waitForEvent('download'),
				visibleButton.click()
			]);
			expect(visibleDownload.suggestedFilename()).toContain('security-logs-visible-');
		}

		const [allDownload] = await Promise.all([
			page.waitForEvent('download'),
			page.getByRole('button', { name: 'Export All CSV' }).click()
		]);
		expect(allDownload.suggestedFilename()).toContain('export.csv');
	});
});
