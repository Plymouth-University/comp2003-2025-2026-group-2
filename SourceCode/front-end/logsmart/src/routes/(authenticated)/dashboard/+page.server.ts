import { redirect, type RequestEvent } from '@sveltejs/kit';
import type { PageServerLoad } from './$types';

export const load: PageServerLoad = async ({ parent, fetch, cookies }: RequestEvent) => {
	const { user } = await parent();

	if (user?.role !== 'admin') {
		throw redirect(303, '/logs-list');
	}

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
