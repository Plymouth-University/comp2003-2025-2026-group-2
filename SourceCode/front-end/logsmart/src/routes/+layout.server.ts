import type { LayoutServerLoad } from './$types';

export const load: LayoutServerLoad = async ({ cookies, fetch }) => {
	const token = cookies.get('ls-token');

	if (!token) {
		return {
			isAuthenticated: false
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
				isAuthenticated: false
			};
		}

		const userData = await response.json();

		return {
			isAuthenticated: true,
			user: userData
		};
	} catch (error) {
		return {
			isAuthenticated: false
		};
	}
};
