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
			members: null
		};
	}

	try {
		const members_resp = await fetch('/api/auth/company/members');
		if (!members_resp.ok) {
			return {
				error: await members_resp.text(),
				members: null
			};
		}

		const invitations_resp = await fetch('/api/auth/invitations/pending');
		if (!invitations_resp.ok) {
			return {
				error: await invitations_resp.text(),
				members: null
			};
		}
		return {
			...(await members_resp.json()),
			invitations: await invitations_resp.json()
		};
	} catch (error) {
		console.error('Error fetching user data:', error);
		return {
			members: null
		};
	}
};
