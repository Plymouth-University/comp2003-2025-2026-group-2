import { redirect } from '@sveltejs/kit';
import type { PageServerLoad } from './$types';

export const load: PageServerLoad = async ({ parent, fetch, cookies, url }) => {
	const { user } = await parent();

	// Only allow company_manager, branch_manager, logsmart_admin, and HQ staff (staff without branch_id)
	const isHQStaff = user?.role === 'staff' && !user?.branch_id;
	if (
		!user ||
		(!['company_manager', 'branch_manager', 'logsmart_admin'].includes(user.role) && !isHQStaff)
	) {
		throw redirect(303, '/dashboard');
	}

	const token = cookies.get('ls-token');

	if (!token) {
		return {
			clockEvents: [],
			user: null,
			branches: [],
			userRole: user.role,
			members: []
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
				branches: [],
				userRole: user.role,
				members: [],
				error: 'Failed to load attendance data'
			};
		}

		const data = await response.json();

		// Fetch branches if user is company_manager
		let branches: Array<{ id: string; name: string; address: string }> = [];
		if (user.role === 'company_manager' || isHQStaff) {
			const branchesResponse = await fetch('/api/auth/company/branches', {
				method: 'GET',
				headers: {
					Authorization: `Bearer ${token}`
				}
			});
			if (branchesResponse.ok) {
				const branchesData = await branchesResponse.json();
				branches = branchesData.branches ?? [];
			}
		}

		// Fetch company members to get user -> branch mapping
		let members: Array<{ id: string; branch_id: string | null }> = [];
		const membersResponse = await fetch('/api/auth/company/members', {
			method: 'GET',
			headers: {
				Authorization: `Bearer ${token}`
			}
		});
		if (membersResponse.ok) {
			const membersData = await membersResponse.json();
			members = (membersData.members ?? []).map((m: any) => ({
				id: m.id,
				branch_id: m.branch_id
			}));
		}

		const d = {
			clockEvents: data.events ?? [],
			user,
			branches,
			userRole: user.role,
			isHQStaff,
			members
		};

		console.log('Loaded attendance data:', d);

		return d;
	} catch (error) {
		console.error('Error fetching attendance data:', error);
		return {
			clockEvents: [],
			user,
			branches: [],
			userRole: user.role,
			members: [],
			error: 'Failed to load attendance data'
		};
	}
};
