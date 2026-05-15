import type { Page } from '@playwright/test';
import { expect } from '@playwright/test';

/**
 * Validates password field behavior across registration, invitation, and password reset flows.
 * Tests password requirements: min 8 chars, uppercase, lowercase, number, special char, and match.
 *
 * @param page - Playwright page object
 * @param scenario - 'empty' | 'mismatch' | 'weak' - which validation to test
 * @param context - 'registration' | 'invitation' | 'reset' - which form context
 *
 * Usage:
 *   // Test empty password in registration
 *   await validatePasswordField(page, 'empty', 'registration');
 *
 *   // Test weak password in password reset
 *   await validatePasswordField(page, 'weak', 'reset');
 */
export async function validatePasswordField(
	page: Page,
	scenario: 'empty' | 'mismatch' | 'weak',
	context: 'registration' | 'invitation' | 'reset'
): Promise<void> {
	const submitButtonText =
		context === 'registration'
			? 'Create Account'
			: context === 'invitation'
				? 'Create Account'
				: 'Set new password';

	if (scenario === 'empty') {
		// Test leaving password field empty
		await page.getByTestId('password-input').press('Tab');
		await page.getByTestId('confirm-password-input').fill('Test123!');
		await expect(page.getByRole('button', { name: submitButtonText })).toBeDisabled();
	} else if (scenario === 'mismatch') {
		// Test password fields with mismatched values
		await page.getByTestId('password-input').fill('Test123!');
		await page.getByTestId('confirm-password-input').fill('Different456!');
		await expect(page.getByRole('button', { name: submitButtonText })).toBeDisabled();
	} else if (scenario === 'weak') {
		// Test weak password (too short, fails requirements)
		await page.getByTestId('password-input').fill('weak');
		await page.getByTestId('confirm-password-input').fill('weak');
		await expect(page.getByRole('button', { name: submitButtonText })).toBeDisabled();
	}
}

/**
 * Validates form field behavior for empty/invalid inputs.
 * Tests that form fields properly validate and disable submit when invalid.
 *
 * @param page - Playwright page object
 * @param fieldName - Role-based field name (e.g., 'First Name', 'Email')
 * @param context - 'registration' | 'invitation' | 'settings' - which form context
 *
 * Usage:
 *   await validateRequiredField(page, 'First Name', 'registration');
 */
export async function validateRequiredField(
	page: Page,
	fieldName: string,
	context: 'registration' | 'invitation' | 'settings'
): Promise<void> {
	const submitButtonText =
		context === 'registration'
			? 'Create Account'
			: context === 'invitation'
				? 'Create Account'
				: 'Save Profile';

	// Clear the field
	const field = page.getByRole('textbox', { name: fieldName });
	await field.clear();
	await field.press('Tab'); // Trigger blur validation

	// Submit button should be disabled
	const submitButton = page.getByRole('button', { name: submitButtonText });
	await expect(submitButton).toBeDisabled();
}
