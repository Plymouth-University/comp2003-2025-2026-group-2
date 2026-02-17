import { redirect, type RequestEvent } from '@sveltejs/kit';
import type { PageServerLoad } from './$types';

export const load: PageServerLoad = async ({ parent, fetch, cookies, url }) => {
	const { user } = await parent();

	if (user?.role !== 'admin' && user?.role !== 'logsmart_admin') {
		throw redirect(303, '/logs-list');
	}

	const token = cookies.get('ls-token');

	if (!token) {
		return {
			clockEvents: [],
			user: null
		};
	}

	try {
		const from = url.searchParams.get('from');
		const to = url.searchParams.get('to');

		let apiUrl = '/api/clock/company';
		const params = new URLSearchParams();
		if (from) params.set('from', from);
		if (to) params.set('to', to);
		const qs = params.toString();
		if (qs) apiUrl += `?${qs}`;

		const response = await fetch(apiUrl, {
			method: 'GET',
			headers: {
				Authorization: `Bearer ${token}`
			}
		});

		if (!response.ok) {
			return {
				clockEvents: [],
				user,
				error: 'Failed to load attendance data'
			};
		}

		const data = await response.json();

		return {
			clockEvents: data.events ?? [],
			user
		};
	} catch (error) {
		console.error('Error fetching attendance data:', error);
		return {
			clockEvents: [],
			user,
			error: 'Failed to load attendance data'
		};
	}
};
