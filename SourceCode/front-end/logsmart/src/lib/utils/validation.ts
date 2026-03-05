/**
 * Validates that a string is a valid CSS color value.
 * Rejects values containing CSS syntax characters that could be used for injection.
 * 
 * Valid formats:
 * - Hex colors: #RGB, #RRGGBB, #RRGGBBAA
 * - RGB/RGBA: rgb(...), rgba(...)
 * - Named colors: red, blue, etc.
 * 
 * @param color - The color string to validate
 * @returns true if the color is valid and safe, false otherwise
 */
export function isValidCSSColor(color: string): boolean {
	// Empty string is valid (means no custom color)
	if (!color || color.trim() === '') {
		return true;
	}

	const trimmed = color.trim();

	// Check for dangerous characters that could be used for CSS injection
	// Semicolon, curly braces, backslash (escape), and @ (at-rules)
	if (/[;{}\\@]/.test(trimmed)) {
		return false;
	}

	// Hex color pattern: #RGB or #RRGGBB or #RRGGBBAA
	if (/^#[0-9a-fA-F]{3}([0-9a-fA-F]{3})?([0-9a-fA-F]{2})?$/.test(trimmed)) {
		return true;
	}

	// RGB/RGBA pattern: rgb(...) or rgba(...)
	// Allows spaces and numbers with commas
	if (/^rgba?\s*\(\s*\d+\s*,\s*\d+\s*,\s*\d+\s*(,\s*[\d.]+\s*)?\)$/.test(trimmed)) {
		return true;
	}

	// Named colors: letters only (no spaces or special chars)
	// This is a conservative check - valid CSS color names contain only letters
	if (/^[a-zA-Z]+$/.test(trimmed)) {
		return true;
	}

	// Hex short format with alpha in older browsers
	if (/^#[0-9a-fA-F]{4}$/.test(trimmed)) {
		return true;
	}

	// If none of the valid formats match, reject it
	return false;
}

/**
 * Sanitizes a color value for safe use in inline styles.
 * Returns the color if valid, or an empty string if invalid.
 * 
 * @param color - The color value to sanitize
 * @returns The sanitized color value or empty string
 */
export function sanitizeColorValue(color: string): string {
	return isValidCSSColor(color) ? color.trim() : '';
}
