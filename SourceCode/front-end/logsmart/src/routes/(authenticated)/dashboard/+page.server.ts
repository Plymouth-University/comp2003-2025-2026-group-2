import { redirect, type RequestEvent } from '@sveltejs/kit';
import type { PageServerLoad } from './$types';

export const load: PageServerLoad = async ({ parent, fetch, cookies }) => {
	const { user } = await parent();

	if (user?.role == 'member') {
		throw redirect(303, '/logs-list');
	}

	const token = cookies.get('ls-token');

	if (!token) {
		return {
			user: null,
			todaysLogs: []
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
				user: null,
				todaysLogs: []
			};
		}

		const userData = await response.json();

		let todaysLogs = [];
		try {
			const logsResponse = await fetch('/api/log-entries/due-today', {
				method: 'GET',
				headers: {
					Authorization: `Bearer ${token}`
				}
			});

			if (logsResponse.ok) {
				todaysLogs = await logsResponse.json();
			}
		} catch (err) {
			console.error('Error fetching due logs:', err);
		}

		return {
			user: userData,
			todaysLogs
		};
	} catch (error) {
		console.error('Error fetching user data:', error);
		return {
			user: null,
			todaysLogs: []
		};
	}
};
