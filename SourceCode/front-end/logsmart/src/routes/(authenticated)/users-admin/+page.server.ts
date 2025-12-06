import { redirect, type RequestEvent } from '@sveltejs/kit';
import type { PageServerLoad } from './$types';

export const load: PageServerLoad = async ({ parent, fetch, cookies }) => {
	const { user } = await parent();

	if (user?.role !== 'admin') {
		throw redirect(303, '/logs-list');
	}

	const token = cookies.get('ls-token');

	if (!token) {
		return {
			members: null
		};
	}

	try {
		const response = await fetch('/api/auth/company/members');
		if (!response.ok) {
			return {
				error: await response.text(),
				members: null
			};
		}

		return await response.json();
	} catch (error) {
		console.error('Error fetching user data:', error);
		return {
			members: null
		};
	}
};
