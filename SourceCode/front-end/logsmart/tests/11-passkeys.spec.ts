import { test, expect } from '@playwright/test';
import { register } from './utils';

test.describe('Passkey Management', () => {
	let userData: any;

	test.beforeEach(async ({ browser }) => {
		userData = await register(browser);
		expect(userData).toBeTruthy();
	});

	test('should register a passkey and login with it', async ({ browser }) => {
		const page = await browser.newPage();

		// 1. Enable Virtual Authenticator
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

		// Login first
		await page.goto('http://localhost:5173/login');
		await page.getByRole('textbox', { name: 'Email' }).fill(userData.email);
		await page.getByRole('textbox', { name: 'Password' }).fill(userData.password);
		await page.getByRole('button', { name: 'Sign in', exact: true }).click();
		await page.waitForURL('**/dashboard');

		// Go to Settings
		await page.getByRole('link', { name: 'Settings' }).click();
		await page.waitForURL('**/settings');

		// Register Passkey
		await page.getByRole('textbox', { name: 'Passkey Name' }).fill('My Virtual Key');
		await page.getByRole('button', { name: 'Add Passkey' }).click();

		// Expect success message
		await expect(page.locator('text=Changes saved successfully!')).toBeVisible();
		await expect(page.locator('text=My Virtual Key')).toBeVisible();

		// Logout
		await page.getByRole('button', { name: 'Logout' }).click();
		await page.waitForURL('**/login');

		// Login with Passkey
		await page.getByRole('textbox', { name: 'Email' }).fill(userData.email);
		await page.getByRole('button', { name: 'Sign in with Passkey' }).click();

		// WebAuthn interaction is simulated automatically by 'automaticPresenceSimulation: true'
		// or effectively by the virtual authenticator interception.

		// Wait for login
		await page.waitForURL('**/dashboard');

		// Remove Authenticator
		await client.send('WebAuthn.removeVirtualAuthenticator', { authenticatorId });
		await client.send('WebAuthn.disable');
		await page.close();
	});

	test('should delete a passkey', async ({ browser }) => {
		const page = await browser.newPage();
		page.on('dialog', (dialog) => dialog.accept());

		// 1. Enable Authenticator
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

		// Login
		await page.goto('http://localhost:5173/login');
		await page.getByRole('textbox', { name: 'Email' }).fill(userData.email);
		await page.getByRole('textbox', { name: 'Password' }).fill(userData.password);
		await page.getByRole('button', { name: 'Sign in', exact: true }).click();
		await page.waitForURL('**/dashboard');

		// Settings
		await page.getByRole('link', { name: 'Settings' }).click();
		await page.waitForURL('**/settings');

		// Register
		await page.getByRole('textbox', { name: 'Passkey Name' }).fill('Key To Delete');
		await page.getByRole('button', { name: 'Add Passkey' }).click();
		await expect(page.locator('text=Key To Delete')).toBeVisible();

		// Delete
		await page.getByRole('button', { name: 'Delete' }).click();

		// Verify gone
		await expect(page.locator('text=Key To Delete')).not.toBeVisible();

		await client.send('WebAuthn.removeVirtualAuthenticator', { authenticatorId });
		await client.send('WebAuthn.disable');
		await page.close();
	});

	test('should register multiple passkeys', async ({ browser }) => {
		const page = await browser.newPage();
		const client = await page.context().newCDPSession(page);
		await client.send('WebAuthn.enable');

		// Track authenticator IDs for cleanup
		const authenticatorIds: string[] = [];

		// Login
		await page.goto('http://localhost:5173/login');
		await page.getByRole('textbox', { name: 'Email' }).fill(userData.email);
		await page.getByRole('textbox', { name: 'Password' }).fill(userData.password);
		await page.getByRole('button', { name: 'Sign in', exact: true }).click();
		await page.waitForURL('**/dashboard');

		// Settings
		await page.getByRole('link', { name: 'Settings' }).click();
		await page.waitForURL('**/settings');

		// Register Key 1 - Create first authenticator (internal)
		const result1 = await client.send('WebAuthn.addVirtualAuthenticator', {
			options: {
				protocol: 'ctap2',
				transport: 'internal',
				hasResidentKey: true,
				hasUserVerification: true,
				isUserVerified: true,
				automaticPresenceSimulation: true
			}
		});
		authenticatorIds.push(result1.authenticatorId);

		await page.getByRole('textbox', { name: 'Passkey Name' }).fill('Key 1');
		await page.getByRole('button', { name: 'Add Passkey' }).click();
		await expect(page.locator('text=Changes saved successfully!')).toBeVisible();
		await expect(page.locator('text=Key 1')).toBeVisible();

		// Register Key 2 - Create second authenticator (USB transport to avoid Chrome's internal limit)
		const result2 = await client.send('WebAuthn.addVirtualAuthenticator', {
			options: {
				protocol: 'ctap2',
				transport: 'usb', // Different transport type
				hasResidentKey: true,
				hasUserVerification: true,
				isUserVerified: true,
				automaticPresenceSimulation: true
			}
		});
		authenticatorIds.push(result2.authenticatorId);

		await page.getByRole('textbox', { name: 'Passkey Name' }).fill('Key 2');
		await page.getByRole('button', { name: 'Add Passkey' }).click();

		// Verify both
		await expect(page.locator('text=Key 1')).toBeVisible();
		await expect(page.locator('text=Key 2')).toBeVisible();

		// Test authentication with Key 1 (internal authenticator)
		await page.getByRole('button', { name: 'Logout' }).click();
		await page.waitForURL('**/login');

		await page.getByRole('textbox', { name: 'Email' }).fill(userData.email);
		await page.getByRole('button', { name: 'Sign in with Passkey' }).click();
		// The first authenticator (internal) should be used automatically
		await page.waitForURL('**/dashboard');

		// Test authentication with Key 2 (USB authenticator)
		// Remove the internal authenticator so USB is used
		await client.send('WebAuthn.removeVirtualAuthenticator', {
			authenticatorId: authenticatorIds[0]
		});

		await page.getByRole('button', { name: 'Logout' }).click();
		await page.waitForURL('**/login');

		await page.getByRole('textbox', { name: 'Email' }).fill(userData.email);
		await page.getByRole('button', { name: 'Sign in with Passkey' }).click();
		// Now the second authenticator (USB) should be used
		await page.waitForURL('**/dashboard');

		// Cleanup remaining authenticator
		await client.send('WebAuthn.removeVirtualAuthenticator', {
			authenticatorId: authenticatorIds[1]
		});
		await client.send('WebAuthn.disable');
		await page.close();
	});
});
