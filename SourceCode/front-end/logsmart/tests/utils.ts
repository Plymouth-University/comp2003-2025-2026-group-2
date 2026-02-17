import { type Browser, expect, type Page } from '@playwright/test';

const MAILHOG_API_URL = process.env.MAILHOG_API_URL || 'http://localhost:8025/api';

interface MailhogEmail {
	ID: string;
	From: {
		Mailbox: string;
		Domain: string;
	};
	To: Array<{
		Mailbox: string;
		Domain: string;
	}>;
	Content: {
		Body: string;
		Headers: Record<string, string[]>;
	};
	Created: string;
}

const getEmailByRecipient = async (email: string): Promise<MailhogEmail | null> => {
	try {
		const response = await fetch(`${MAILHOG_API_URL}/v2/messages`);
		if (!response.ok) {
			console.error(`Mailhog API error: ${response.status}`);
			return null;
		}

		const text = await response.text();
		if (!text) {
			console.warn('Mailhog API returned empty response');
			return null;
		}

		const data = JSON.parse(text);
		const emails = data.items || [];

		return (
			emails.find((e: MailhogEmail) =>
				e.To?.some((to) => `${to.Mailbox}@${to.Domain}`.toLowerCase() === email.toLowerCase())
			) || null
		);
	} catch (error) {
		console.error('Failed to fetch emails from Mailhog:', error);
		return null;
	}
};

const getInvitationToken = async (email: string, maxAttempts = 30): Promise<string | null> => {
	for (let i = 0; i < maxAttempts; i++) {
		const mailhogEmail = await getEmailByRecipient(email);

		if (mailhogEmail) {
			let body = mailhogEmail.Content.Body;

			body = body.replace(/=\r?\n/g, '');
			body = body.replace(/=3D/g, '=');

			const tokenMatch = body.match(/token=([a-zA-Z0-9_-]+)/);
			if (tokenMatch) {
				return tokenMatch[1];
			}
		}

		await new Promise((resolve) => setTimeout(resolve, 500));
	}

	return null;
};

const decodeMailBody = (email: MailhogEmail): string => {
	const encoding = email.Content.Headers['Content-Transfer-Encoding']?.[0]?.toLowerCase();
	let body = email.Content.Body;

	if (encoding === 'base64') {
		try {
			body = Buffer.from(body, 'base64').toString('utf-8');
		} catch (e) {
			console.error('Failed to decode base64 email body:', e);
		}
	}

	body = body.replace(/=\r?\n/g, '');
	body = body.replace(/=3D/g, '=');

	return body;
};

const getPasswordResetToken = async (email: string, maxAttempts = 60): Promise<string | null> => {
	for (let i = 0; i < maxAttempts; i++) {
		const mailhogEmail = await getEmailByRecipient(email);

		if (mailhogEmail) {
			const body = decodeMailBody(mailhogEmail);

			const tokenMatch = body.match(/token=([a-zA-Z0-9_-]+)/);
			if (tokenMatch) {
				return tokenMatch[1];
			}
		}

		await new Promise((resolve) => setTimeout(resolve, 1000));
	}

	return null;
};

const getBranchDeletionToken = async (email: string, maxAttempts = 30): Promise<string | null> => {
	for (let i = 0; i < maxAttempts; i++) {
		const mailhogEmail = await getEmailByRecipient(email);

		if (mailhogEmail) {
			const body = decodeMailBody(mailhogEmail);

			const tokenMatch = body.match(/token=([a-zA-Z0-9_-]+)/);
			if (tokenMatch) {
				return tokenMatch[1];
			}
		}

		await new Promise((resolve) => setTimeout(resolve, 500));
	}

	return null;
};

const clearMailhogEmails = async (): Promise<void> => {
	try {
		await fetch(`${MAILHOG_API_URL}/v1/messages`, { method: 'DELETE' });
	} catch (error) {
		console.error('Failed to clear Mailhog emails:', error);
	}
};

const requestPasswordResetToken = async (
	browser: Browser,
	email: string
): Promise<string | null> => {
	await clearMailhogEmails();
	const page = await browser.newPage();
	await page.goto('http://localhost:5173/reset-password');
	await page.getByRole('textbox', { name: 'Email' }).fill(email);
	await page.getByRole('button', { name: 'Send Reset Link' }).click();
	await page.waitForTimeout(2000);
	await page.close();

	return await getPasswordResetToken(email, 60);
};

const register = async (browser: Browser, close = true) => {
	const slug = `${Date.now()}-${Math.floor(Math.random() * 1_000_000)}`;
	const companyName = `TestCompany-${slug}`;
	const firstName = `Test-${slug}`;
	const lastName = `User-${slug}`;
	const email = `testuser+${slug}@logsmart.app`;
	const password = `Test${slug}!A`;
	const page = await browser.newPage();
	await page.goto('http://localhost:5173/');
	await page.getByRole('link', { name: 'Register Company' }).click();
	await page.waitForURL('**/register-company');
	await page.getByRole('textbox', { name: 'Company Name' }).click();
	await page.getByRole('textbox', { name: 'Company Name' }).fill(companyName);
	await page.getByRole('textbox', { name: 'Company Name' }).press('Tab');
	await page
		.getByRole('textbox', { name: 'Company Address' })
		.fill('TestAddress1, ABC\nSecond Line,\n2!');
	await page.getByRole('button', { name: 'Next Step' }).click();
	await page.getByRole('textbox', { name: 'First Name' }).click();
	await page.getByRole('textbox', { name: 'First Name' }).fill(firstName);
	await page.getByRole('textbox', { name: 'First Name' }).press('Tab');
	await page.getByRole('textbox', { name: 'Last Name' }).fill(lastName);
	await page.getByRole('textbox', { name: 'Last Name' }).press('Tab');
	await page.getByRole('textbox', { name: 'Email' }).fill(email);
	await page.getByRole('textbox', { name: 'Email' }).press('Tab');
	await page.getByRole('textbox', { name: 'Password Show password', exact: true }).fill(password);
	await page.getByRole('textbox', { name: 'Confirm Password Show password' }).click();
	await page.getByRole('textbox', { name: 'Confirm Password Show password' }).fill(password);
	await page.getByRole('button', { name: 'Create Account' }).click();
	if (
		await page
			.locator('body')
			.textContent()
			.then((text) => text?.includes('Email already exists'))
	) {
		console.error('Email already exists. Cannot register the same email again.');
		return null;
	}
	await page.waitForURL('**/dashboard');
	if (close) {
		await page.close();
		return { companyName, firstName, lastName, email, password };
	}
	return { companyName, firstName, lastName, email, password, page };
};

const createBranch = async (page: Page, name: string, address: string): Promise<void> => {
	await page.getByRole('link', { name: 'Branches' }).click();
	await page.waitForURL('**/branches');
	await page.getByRole('textbox', { name: 'Branch Name' }).fill(name);
	await page.getByRole('textbox', { name: 'Address' }).fill(address);
	await page.locator('form > div.search-container.relative.flex-1 > div > button').first().click();
	await page.getByRole('button', { name: 'ADD BRANCH' }).click();
	await expect(page.getByText(name)).toBeVisible();
};

const sendInvitation = async (
	browser: Browser,
	admin: { email: string; password: string },
	email: string,
	role: string = 'staff',
	branchName?: string
): Promise<string | null> => {
	const page = await browser.newPage();
	await page.goto('http://localhost:5173/login');
	await page.getByRole('textbox', { name: 'Email' }).fill(admin.email);
	await page.getByRole('textbox', { name: 'Password' }).fill(admin.password);
	await page.getByRole('button', { name: 'Sign in', exact: true }).click();
	await page.waitForURL('**/dashboard');
	await page.getByRole('link', { name: 'Users' }).click();
	await page.waitForURL('**/users-admin');
	await page.getByRole('button', { name: '➕' }).click();
	await page.getByRole('textbox', { name: "New user's email" }).fill(email);

	console.log(`[sendInvitation] Selecting role: ${role}`);
	await page.locator('#invite-role').selectOption(role);

	if (branchName) {
		console.log(`[sendInvitation] Selecting branch: ${branchName}`);
		const select = page.locator('#invite-branch');
		await expect(select).toBeVisible();
		await page.waitForTimeout(500);
		try {
			await select.selectOption({ label: branchName });
		} catch {
			try {
				const options = await select.locator('option').all();
				for (let i = 0; i < options.length; i++) {
					const text = await options[i].textContent();
					if (text && text.includes(branchName)) {
						await select.selectOption({ index: i });
						break;
					}
				}
			} catch (e) {
				console.log(`[sendInvitation] Could not select branch: ${e}`);
			}
		}
	} else if (role !== 'staff') {
		const select = page.locator('#invite-branch');
		await expect(select).toBeVisible();
		const options = await select.locator('option').all();
		if (options.length > 1) {
			await select.selectOption({ index: 1 });
		}
	}

	console.log(`[sendInvitation] Clicking Send Invite`);
	await page.getByRole('button', { name: 'Send Invite' }).click();

	await page.waitForTimeout(1000);
	await page.close();

	console.log(`[sendInvitation] Fetching token from Mailhog for ${email}`);
	return await getInvitationToken(email);
};

const acceptInvitation = async (
	page: Page,
	token: string,
	firstName: string,
	lastName: string,
	password: string,
	waitFor: string = '**/dashboard'
): Promise<boolean> => {
	await page.goto(`http://localhost:5173/accept-invitation?token=${token}`);
	await page.waitForURL('**/accept-invitation**');

	// Check if there is an error message
	const errorLocator = page.locator('.text-red-600');
	if (await errorLocator.isVisible()) {
		const errorText = await errorLocator.textContent();
		console.error(`[acceptInvitation] Error on page: ${errorText}`);
		return false;
	}

	await page.getByRole('button', { name: 'Accept Invitation' }).click();

	await page.getByRole('textbox', { name: 'First Name' }).fill(firstName);
	await page.getByRole('textbox', { name: 'Last Name' }).fill(lastName);

	await page.getByRole('textbox', { name: 'Password Show password', exact: true }).fill(password);
	await page.getByRole('textbox', { name: 'Confirm Password' }).fill(password);
	await page.getByRole('button', { name: 'Create Account' }).click();

	await page.waitForURL(waitFor, { timeout: 10000 });
	return page.url().includes(waitFor.replace('**', ''));
};

const sendInvitationOnPage = async (
	page: Page,
	email: string,
	role: string = 'staff',
	branchName?: string
): Promise<string | null> => {
	await page.getByRole('link', { name: 'Users' }).click();
	await page.waitForURL('**/users-admin');

	await page.getByRole('button', { name: '➕' }).click();
	await page.waitForLoadState('networkidle');
	await page.getByRole('textbox', { name: "New user's email" }).fill(email);

	await page.locator('#invite-role').selectOption(role);

	if (branchName) {
		const select = page.locator('#invite-branch');
		await expect(select).toBeVisible();
		await page.waitForTimeout(500);
		try {
			await select.selectOption({ label: branchName });
		} catch {
			try {
				const options = await select.locator('option').all();
				for (let i = 0; i < options.length; i++) {
					const text = await options[i].textContent();
					if (text && text.includes(branchName)) {
						await select.selectOption({ index: i });
						break;
					}
				}
			} catch (e) {
				console.log(`[sendInvitationOnPage] Could not select branch: ${e}`);
			}
		}
	} else if (role !== 'staff') {
		const select = page.locator('#invite-branch');
		await expect(select).toBeVisible();
		const options = await select.locator('option').all();
		if (options.length > 1) {
			await select.selectOption({ index: 1 });
		}
	}

	await page.getByRole('button', { name: 'Send Invite' }).click();

	return await getInvitationToken(email);
};

export {
	register,
	createBranch,
	getEmailByRecipient,
	getInvitationToken,
	getBranchDeletionToken,
	getPasswordResetToken,
	clearMailhogEmails,
	requestPasswordResetToken,
	sendInvitation,
	sendInvitationOnPage,
	acceptInvitation
};
