import { test, expect } from '@playwright/test';
import { register, clearMailhogEmails, getCompanyDeletionToken } from './utils';

let adminCreds: {
	email: string;
	password: string;
	companyName: string;
	firstName: string;
	lastName: string;
};

test.beforeAll(async ({ browser }) => {
	await clearMailhogEmails();
	const creds = await register(browser);
	if (!creds) throw new Error('Failed to register admin user');
	adminCreds = creds;
});

test.beforeEach(async ({ page }) => {
	await page.goto('http://localhost:5173/');
	await page.getByRole('link', { name: 'Login' }).click();
	await page.getByRole('textbox', { name: 'Email' }).fill(adminCreds.email);
	await page.getByRole('textbox', { name: 'Password' }).fill(adminCreds.password);
	await page.getByRole('button', { name: 'Sign in', exact: true }).click();
	await page.waitForURL('**/dashboard');
});

test.describe('Confirm Company Deletion Page', () => {
	test('shows_error_when_missing_params', async ({ page }) => {
		await page.goto('http://localhost:5173/confirm-company-deletion');
		await expect(page.getByText('Missing company ID or confirmation token')).toBeVisible({
			timeout: 10000
		});
	});

	test('shows_error_when_invalid_token', async ({ page }) => {
		await page.goto(
			'http://localhost:5173/confirm-company-deletion?company_id=test-123&token=invalid-token'
		);
		await expect(
			page.getByText(
				/Failed to delete company|The token may have expired|Invalid or expired|Company not found/
			)
		).toBeVisible({ timeout: 10000 });
	});
});

test.describe('Complete Company Deletion Flow', () => {
	test('full_deletion_flow_with_email_confirmation', async ({ browser }) => {
		await clearMailhogEmails();
		const creds = await register(browser);
		if (!creds) throw new Error('Failed to register admin user');

		const page = await browser.newPage();
		await page.goto('http://localhost:5173/login');
		await page.getByRole('textbox', { name: 'Email' }).fill(creds.email);
		await page.getByRole('textbox', { name: 'Password' }).fill(creds.password);
		await page.getByRole('button', { name: 'Sign in', exact: true }).click();
		await page.waitForURL('**/dashboard');

		await page.getByRole('link', { name: 'Company Settings' }).click();
		await page.waitForURL('**/company-settings');
		await page.reload();
		await page.waitForURL('**/company-settings');

		await page.getByRole('button', { name: /Export Company Data|Re-export Company Data/ }).click();
		await page.waitForTimeout(2000);

		const deleteButton = page.getByRole('button', { name: 'Delete Company' });
		await expect(deleteButton).toBeEnabled({ timeout: 10000 });

		page.on('dialog', async (dialog) => {
			await dialog.accept();
		});

		await deleteButton.click();
		await page.waitForTimeout(2000);

		const tokenData = await getCompanyDeletionToken(creds.email);
		expect(tokenData).toBeTruthy();

		const { token, companyId } = JSON.parse(tokenData!);

		const confirmPage = await browser.newPage();
		await confirmPage.goto(
			`http://localhost:5173/confirm-company-deletion?company_id=${companyId}&token=${token}`
		);
		await confirmPage.waitForURL('**/confirm-company-deletion**');

		await expect(confirmPage.locator('body')).toContainText('Confirm Company Deletion');

		await expect(
			confirmPage.getByRole('textbox', { name: 'Type company name to confirm' })
		).toBeVisible({ timeout: 10000 });
		await confirmPage
			.getByRole('textbox', { name: 'Type company name to confirm' })
			.fill(creds.companyName);

		await confirmPage.getByRole('button', { name: 'Delete Company' }).click();

		await expect(confirmPage.locator('body')).toContainText('Success');

		await page.close();
		await confirmPage.close();
	});

	test('user_cannot_login_after_company_deletion', async ({ browser, page }) => {
		await clearMailhogEmails();
		const creds = await register(browser);
		if (!creds) throw new Error('Failed to register admin user');

		await page.goto('http://localhost:5173/login');
		await page.getByRole('textbox', { name: 'Email' }).fill(creds.email);
		await page.getByRole('textbox', { name: 'Password' }).fill(creds.password);
		await page.getByRole('button', { name: 'Sign in', exact: true }).click();
		await page.waitForURL('**/dashboard');

		await page.getByRole('link', { name: 'Company Settings' }).click();
		await page.waitForURL('**/company-settings');
		await page.reload();
		await page.waitForURL('**/company-settings');

		await page.getByRole('button', { name: /Export Company Data|Re-export Company Data/ }).click();
		await page.waitForTimeout(2000);

		const deleteButton = page.getByRole('button', { name: 'Delete Company' });
		await expect(deleteButton).toBeEnabled({ timeout: 10000 });

		page.on('dialog', async (dialog) => {
			await dialog.accept();
		});

		await deleteButton.click();
		await page.waitForTimeout(2000);
		await page.close();

		const tokenData = await getCompanyDeletionToken(creds.email);
		expect(tokenData).toBeTruthy();

		const { token, companyId } = JSON.parse(tokenData!);

		const confirmPage = await browser.newPage();
		await confirmPage.goto(
			`http://localhost:5173/confirm-company-deletion?company_id=${companyId}&token=${token}`
		);
		await confirmPage.waitForURL('**/confirm-company-deletion**');

		await expect(
			confirmPage.getByRole('textbox', { name: 'Type company name to confirm' })
		).toBeVisible({ timeout: 10000 });
		await confirmPage
			.getByRole('textbox', { name: 'Type company name to confirm' })
			.fill(creds.companyName);

		await confirmPage.getByRole('button', { name: 'Delete Company' }).click();
		await expect(confirmPage.locator('body')).toContainText('Success');

		await confirmPage.close();

		const loginPage = await browser.newPage();
		await loginPage.goto('http://localhost:5173/login');
		await loginPage.getByRole('textbox', { name: 'Email' }).fill(creds.email);
		await loginPage.getByRole('textbox', { name: 'Password' }).fill(creds.password);
		await loginPage.getByRole('button', { name: 'Sign in', exact: true }).click();

		await expect(loginPage.locator('body')).toContainText('Your company has been deleted');

		await loginPage.close();
	});

	test('api_calls_blocked_after_company_deletion', async ({ browser }) => {
		await clearMailhogEmails();
		const creds = await register(browser);
		if (!creds) throw new Error('Failed to register admin user');

		const page = await browser.newPage();
		await page.goto('http://localhost:5173/login');
		await page.getByRole('textbox', { name: 'Email' }).fill(creds.email);
		await page.getByRole('textbox', { name: 'Password' }).fill(creds.password);
		await page.getByRole('button', { name: 'Sign in', exact: true }).click();
		await page.waitForURL('**/dashboard');

		const token = await page.evaluate(() => {
			const cookies = document.cookie.split(';');
			for (const cookie of cookies) {
				if (cookie.trim().startsWith('ls-token=')) {
					return cookie.trim().substring('ls-token='.length);
				}
			}
			return null;
		});

		await page.getByRole('link', { name: 'Company Settings' }).click();
		await page.waitForURL('**/company-settings');
		await page.reload();
		await page.waitForURL('**/company-settings');

		await page.getByRole('button', { name: /Export Company Data|Re-export Company Data/ }).click();
		await page.waitForTimeout(2000);

		const deleteButton = page.getByRole('button', { name: 'Delete Company' });
		await expect(deleteButton).toBeEnabled({ timeout: 10000 });

		page.on('dialog', async (dialog) => {
			await dialog.accept();
		});

		await deleteButton.click();
		await page.waitForTimeout(2000);

		const tokenData = await getCompanyDeletionToken(creds.email);
		expect(tokenData).toBeTruthy();

		const { token: delToken, companyId } = JSON.parse(tokenData!);

		const confirmPage = await browser.newPage();
		await confirmPage.goto(
			`http://localhost:5173/confirm-company-deletion?company_id=${companyId}&token=${delToken}`
		);
		await confirmPage.waitForURL('**/confirm-company-deletion**');

		await expect(
			confirmPage.getByRole('textbox', { name: 'Type company name to confirm' })
		).toBeVisible({ timeout: 10000 });
		await confirmPage
			.getByRole('textbox', { name: 'Type company name to confirm' })
			.fill(creds.companyName);

		await confirmPage.getByRole('button', { name: 'Delete Company' }).click();
		await expect(confirmPage.locator('body')).toContainText('Success');

		await confirmPage.close();

		const apiResponse = await fetch('http://localhost:6767/auth/me', {
			headers: {
				Authorization: `Bearer ${token}`
			}
		});
		const apiBody = await apiResponse.json();
		expect(apiResponse.status).toBe(401);
		expect(apiBody.error).toContain('deleted');

		await page.close();
	});

	test('passkey_login_blocked_after_company_deletion', async ({ browser }) => {
		await clearMailhogEmails();
		const creds = await register(browser);
		if (!creds) throw new Error('Failed to register admin user');

		const page = await browser.newPage();
		await page.addInitScript(() => {
			if (window.PublicKeyCredential) {
				window.PublicKeyCredential.isConditionalMediationAvailable = async () => false;
			}
		});

		const client = await page.context().newCDPSession(page);
		await client.send('WebAuthn.enable');
		const result = await client.send('WebAuthn.addVirtualAuthenticator', {
			options: {
				protocol: 'ctap2',
				transport: 'internal',
				hasResidentKey: true,
				hasUserVerification: true,
				isUserVerified: true,
				automaticPresenceSimulation: true
			}
		});
		const authenticatorId = result.authenticatorId;

		await page.goto('http://localhost:5173/login');
		await page.getByRole('textbox', { name: 'Email' }).fill(creds.email);
		await page.getByRole('textbox', { name: 'Password' }).fill(creds.password);
		await page.getByRole('button', { name: 'Sign in', exact: true }).click();
		await page.waitForURL('**/dashboard');

		await page.getByRole('link', { name: 'Settings', exact: true }).click();
		await page.waitForURL('**/settings');

		await page.getByRole('textbox', { name: 'Passkey Name' }).fill('Test Passkey');
		await page.getByRole('button', { name: 'Add Passkey' }).click();
		await expect(page.locator('text=Changes saved successfully!')).toBeVisible({ timeout: 10000 });

		await page.getByRole('button', { name: 'Logout' }).click();
		await page.waitForURL('**/login');

		await page.getByRole('link', { name: 'Company Settings' }).click();
		await page.waitForURL('**/company-settings');
		await page.reload();
		await page.waitForURL('**/company-settings');

		await page.getByRole('button', { name: /Export Company Data|Re-export Company Data/ }).click();
		await page.waitForTimeout(2000);

		const deleteButton = page.getByRole('button', { name: 'Delete Company' });
		await expect(deleteButton).toBeEnabled({ timeout: 10000 });

		page.on('dialog', async (dialog) => {
			await dialog.accept();
		});

		await deleteButton.click();
		await page.waitForTimeout(2000);

		const tokenData = await getCompanyDeletionToken(creds.email);
		expect(tokenData).toBeTruthy();

		const { token: delToken, companyId } = JSON.parse(tokenData!);

		const confirmPage = await browser.newPage();
		await confirmPage.goto(
			`http://localhost:5173/confirm-company-deletion?company_id=${companyId}&token=${delToken}`
		);
		await confirmPage.waitForURL('**/confirm-company-deletion**');

		await expect(
			confirmPage.getByRole('textbox', { name: 'Type company name to confirm' })
		).toBeVisible({ timeout: 10000 });
		await confirmPage
			.getByRole('textbox', { name: 'Type company name to confirm' })
			.fill(creds.companyName);

		await confirmPage.getByRole('button', { name: 'Delete Company' }).click();
		await expect(confirmPage.locator('body')).toContainText('Success');

		await confirmPage.close();

		const loginPage = await browser.newPage();
		await loginPage.addInitScript(() => {
			if (window.PublicKeyCredential) {
				window.PublicKeyCredential.isConditionalMediationAvailable = async () => false;
			}
		});

		const loginClient = await loginPage.context().newCDPSession(loginPage);
		await loginClient.send('WebAuthn.enable');
		await loginClient.send('WebAuthn.addVirtualAuthenticator', {
			options: {
				protocol: 'ctap2',
				transport: 'internal',
				hasResidentKey: true,
				hasUserVerification: true,
				isUserVerified: true,
				automaticPresenceSimulation: true
			}
		});

		await loginPage.goto('http://localhost:5173/login');
		await loginPage.getByRole('textbox', { name: 'Email' }).fill(creds.email);

		const passkeyBtn = loginPage.getByRole('button', { name: 'Sign in with Passkey' });
		await passkeyBtn.waitFor({ state: 'visible' });
		await passkeyBtn.click();

		await loginPage.waitForTimeout(2000);
		const currentUrl = loginPage.url();
		expect(currentUrl).not.toContain('dashboard');
		expect(currentUrl).toContain('login');

		await loginClient.send('WebAuthn.removeVirtualAuthenticator');
		await loginClient.send('WebAuthn.disable');
		await loginPage.close();
		await page.close();
	});

	test('oauth_login_blocked_after_company_deletion', async ({ browser }) => {
		test.skip(!!process.env.CI, 'Skipping OAuth test on CI');
		await clearMailhogEmails();
		const creds = await register(browser);
		if (!creds) throw new Error('Failed to register admin user');

		const page = await browser.newPage();
		await page.goto('http://localhost:5173/login');
		await page.getByRole('textbox', { name: 'Email' }).fill(creds.email);
		await page.getByRole('textbox', { name: 'Password' }).fill(creds.password);
		await page.getByRole('button', { name: 'Sign in', exact: true }).click();
		await page.waitForURL('**/dashboard');

		await page.goto('http://localhost:5173/settings');
		await page.waitForLoadState('networkidle');
		await page.getByRole('button', { name: 'Link Google Account' }).click();
		await page.waitForURL(/localhost:8080/, { timeout: 10000 });

		await page.fill('input[name="subject"], input#subject, input[type="text"]', creds.email);
		const claimsJson = JSON.stringify({
			email: creds.email,
			email_verified: true,
			given_name: creds.firstName,
			family_name: creds.lastName,
			picture: 'https://example.com/avatar.jpg'
		});
		await page.fill('textarea', claimsJson);
		await page.getByRole('button', { name: /sign-in/i }).first().click();
		await page.waitForURL('**/settings', { timeout: 10000 });

		await page.getByRole('button', { name: /logout/i }).click();
		await page.waitForURL('**/login');

		await page.getByRole('link', { name: 'Company Settings' }).click();
		await page.waitForURL('**/company-settings');
		await page.reload();
		await page.waitForURL('**/company-settings');

		await page.getByRole('button', { name: /Export Company Data|Re-export Company Data/ }).click();
		await page.waitForTimeout(2000);

		const deleteButton = page.getByRole('button', { name: 'Delete Company' });
		await expect(deleteButton).toBeEnabled({ timeout: 10000 });

		page.on('dialog', async (dialog) => {
			await dialog.accept();
		});

		await deleteButton.click();
		await page.waitForTimeout(2000);

		const tokenData = await getCompanyDeletionToken(creds.email);
		expect(tokenData).toBeTruthy();

		const { token: delToken, companyId } = JSON.parse(tokenData!);

		const confirmPage = await browser.newPage();
		await confirmPage.goto(
			`http://localhost:5173/confirm-company-deletion?company_id=${companyId}&token=${delToken}`
		);
		await confirmPage.waitForURL('**/confirm-company-deletion**');

		await expect(
			confirmPage.getByRole('textbox', { name: 'Type company name to confirm' })
		).toBeVisible({ timeout: 10000 });
		await confirmPage
			.getByRole('textbox', { name: 'Type company name to confirm' })
			.fill(creds.companyName);

		await confirmPage.getByRole('button', { name: 'Delete Company' }).click();
		await expect(confirmPage.locator('body')).toContainText('Success');

		await confirmPage.close();

		const loginPage = await browser.newPage();
		await loginPage.goto('http://localhost:5173/login');
		await loginPage.getByRole('button', { name: 'Sign in with Google' }).click();
		await loginPage.waitForURL(/localhost:8080/, { timeout: 10000 });

		await loginPage.fill('input[name="subject"], input#subject, input[type="text"]', creds.email);
		const loginClaimsJson = JSON.stringify({
			email: creds.email,
			email_verified: true,
			given_name: creds.firstName,
			family_name: creds.lastName,
			picture: 'https://example.com/avatar.jpg'
		});
		await loginPage.fill('textarea', loginClaimsJson);
		await loginPage.getByRole('button', { name: /sign-in/i }).first().click();

		await loginPage.waitForTimeout(2000);
		const currentUrl = loginPage.url();
		expect(currentUrl).not.toContain('dashboard');
		expect(currentUrl).toContain('login');

		await loginPage.close();
		await page.close();
	});
});
