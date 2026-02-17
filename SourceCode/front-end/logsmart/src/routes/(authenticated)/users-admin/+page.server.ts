import { redirect } from '@sveltejs/kit';
import type { PageServerLoad } from './$types';

export const load: PageServerLoad = async ({ parent, fetch, cookies }) => {
	const { user } = await parent();

	// Allow readonly HQ (staff with no branch) to view users
	const isReadonlyHQ = user?.role === 'staff' && !user?.branch_id;
	if (user?.role === 'staff' && !isReadonlyHQ) {
		throw redirect(303, '/logs-list');
	}

	const token = cookies.get('ls-token');

	if (!token) {
		return {
			members: null,
			invitations: null,
			user: null
		};
	}

	try {
		const members_resp = await fetch('/api/auth/company/members');
		if (!members_resp.ok) {
			console.error('Failed to fetch members:', members_resp.status, await members_resp.text());
			return {
				error: 'Failed to fetch members',
				members: null,
				invitations: null,
				user: user
			};
		}

		const members_data = await members_resp.json();
		console.log(`Fetched ${members_data.members?.length} members for user ${user?.email}`);

		const invitations_resp = await fetch('/api/auth/invitations/pending');
		if (!invitations_resp.ok) {
			const errorText = await invitations_resp.text();
			console.error('Failed to fetch invitations:', invitations_resp.status, errorText);
			return {
				error: 'Failed to fetch invitations',
				members: members_data.members,
				invitations: null,
				branches: [],
				user: user
			};
		}
		const invitations_data = await invitations_resp.json();

		const branches_resp = await fetch('/api/auth/company/branches');
		const branches_data = branches_resp.ok ? await branches_resp.json() : { branches: [] };

		return {
			members: members_data.members || [],
			invitations: invitations_data.invitations || [],
			branches: branches_data.branches || [],
			user: user
		};
	} catch (error) {
		console.error('Error fetching user data:', error);
		return {
			members: null,
			invitations: null,
			user: user
		};
	}
};
