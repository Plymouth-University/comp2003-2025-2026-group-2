import { redirect } from '@sveltejs/kit';
import type { PageServerLoad } from './$types';

export const load: PageServerLoad = async ({ parent, fetch, cookies }) => {
	const { user } = await parent();

	// Allow readonly HQ (staff with no branch) to access dashboard
	const isReadonlyHQ = user?.role === 'staff' && !user?.branch_id;
	if (user?.role === 'staff' && !isReadonlyHQ) {
		throw redirect(303, '/logs-list');
	}

	const token = cookies.get('ls-token');

	if (!token) {
		return {
			user: null,
			todaysLogs: [],
			clockStatus: null
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
				todaysLogs: [],
				clockStatus: null
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

		let clockStatus = null;
		try {
			const clockResponse = await fetch('/api/clock/status', {
				method: 'GET',
				headers: {
					Authorization: `Bearer ${token}`
				}
			});

			if (clockResponse.ok) {
				clockStatus = await clockResponse.json();
			}
		} catch (err) {
			console.error('Error fetching clock status:', err);
		}

		return {
			user: userData,
			todaysLogs,
			clockStatus
		};
	} catch (error) {
		console.error('Error fetching user data:', error);
		return {
			user: null,
			todaysLogs: [],
			clockStatus: null
		};
	}
};
