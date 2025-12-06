import type { PageServerLoad, Actions } from './$types';
import { fail, type RequestEvent } from '@sveltejs/kit';

export const load: PageServerLoad = async ({ fetch, cookies }) => {
	const token = cookies.get('ls-token');

	if (!token) {
		return {
			user: null
		};
	}

	try {
		const response = await fetch('/api/auth/me', {
			method: 'GET',
			headers: {
				Authorization: `Bearer ${token}`
			}
		});

		if (!response.ok) {
			return {
				user: null
			};
		}

		const userData = await response.json();

		return {
			user: userData
		};
	} catch (error) {
		console.error('Error fetching user data:', error);
		return {
			user: null
		};
	}
};

export const actions: Actions = {
	updateProfile: async ({ request, fetch, cookies }) => {
		const token = cookies.get('ls-token');

		if (!token) {
			return fail(401, { message: 'Unauthorized' });
		}

		const formData = await request.formData();
		const firstName = formData.get('firstName')?.toString();
		const lastName = formData.get('lastName')?.toString();

		try {
			const response = await fetch('/api/auth/profile', {
				method: 'PUT',
				headers: {
					Authorization: `Bearer ${token}`,
					'Content-Type': 'application/json'
				},
				body: JSON.stringify({
					first_name: firstName,
					last_name: lastName
				})
			});

			// Read response body once
			const text = await response.text();

			console.log('Profile update response status:', response.status);
			console.log('Profile update response body:', text);

			if (!response.ok) {
				let errorMessage = 'Failed to update profile';
				try {
					if (text) {
						const error = JSON.parse(text);
						errorMessage = error.message || error.error || errorMessage;
						console.log('Parsed error:', error);
					}
				} catch {
					// Use text as-is if not JSON
					errorMessage = text || errorMessage;
				}
				console.error('Profile update failed:', errorMessage);
				return fail(response.status, {
					message: errorMessage,
					success: false
				});
			}

			// Handle successful response
			if (text) {
				try {
					const data = JSON.parse(text);
					console.log('Profile update response:', data);
				} catch {
					// Non-JSON response is OK for updates
					console.log('Profile update response (text):', text);
				}
			}

			return {
				success: true,
				message: 'Profile updated successfully'
			};
		} catch (error) {
			console.error('Error updating profile:', error);
			return fail(500, {
				message: 'Network error',
				success: false
			});
		}
	},

	changePassword: async ({ request, fetch, cookies }) => {
		const token = cookies.get('ls-token');

		if (!token) {
			return fail(401, { message: 'Unauthorized' });
		}

		const formData = await request.formData();
		const email = formData.get('email')?.toString();

		if (!email) {
			return fail(400, {
				message: 'Email is required',
				success: false
			});
		}

		try {
			const response = await fetch('/api/auth/password/request-reset', {
				method: 'POST',
				headers: {
					'Content-Type': 'application/json'
				},
				body: JSON.stringify({
					email
				})
			});

			// Read response body once
			const text = await response.text();

			console.log('Password reset request response status:', response.status);
			console.log('Password reset request response body:', text);

			if (!response.ok) {
				let errorMessage = 'Failed to send password reset email';
				try {
					if (text) {
						const error = JSON.parse(text);
						errorMessage = error.message || error.error || errorMessage;
						console.log('Parsed error:', error);
					}
				} catch {
					// Use text as-is if not JSON
					errorMessage = text || errorMessage;
				}
				console.error('Password reset request failed:', errorMessage);
				return fail(response.status, {
					message: errorMessage,
					success: false
				});
			}

			// Handle successful response
			if (text) {
				try {
					const data = JSON.parse(text);
					console.log('Password reset request response:', data);
				} catch {
					// Non-JSON response is OK
					console.log('Password reset request response (text):', text);
				}
			}

			return {
				success: true,
				message: 'Password reset link sent to your email'
			};
		} catch (error) {
			console.error('Error requesting password reset:', error);
			return fail(500, {
				message: 'Network error',
				success: false
			});
		}
	}
};
