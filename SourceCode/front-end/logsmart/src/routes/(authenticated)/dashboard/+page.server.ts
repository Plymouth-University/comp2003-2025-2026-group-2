import type { PageServerLoad } from './$types';

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
