import { redirect } from '@sveltejs/kit';
import type { PageServerLoad } from './$types';
import type { components } from '../../../lib/api-types';
type UserResponse = components['schemas']['UserResponse'];

/**
 * Calculate default date range for last 7 days
 * Returns timestamps with sevenDaysAgo at start of day (00:00:00 UTC)
 * and today at end of day (23:59:59.999 UTC)
 */
function getDefaultDateRange() {
	const now = new Date();
	const sevenDaysAgo = new Date(now.getTime() - 7 * 24 * 60 * 60 * 1000);

	// Set sevenDaysAgo to start of day (00:00:00 UTC)
	sevenDaysAgo.setUTCHours(0, 0, 0, 0);

	// Set now to end of day (23:59:59.999 UTC)
	now.setUTCHours(23, 59, 59, 999);

	return {
		from: sevenDaysAgo.toISOString(),
		to: now.toISOString()
	};
}
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
		let from = url.searchParams.get('from');
		let to = url.searchParams.get('to');
		const cursor = url.searchParams.get('cursor');
		let appliedDefaults = false;

		// If no date range specified, use default (last 7 days)
		if (!from || !to) {
			const defaultRange = getDefaultDateRange();
			from = defaultRange.from;
			to = defaultRange.to;
			appliedDefaults = true;
		}

		let apiUrl = '/api/clock/company';
		const params = new URLSearchParams();
		if (from) params.set('from', from);
		if (to) params.set('to', to);
		if (cursor) params.set('cursor', cursor);
		params.set('limit', '25');
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
				defaultFrom: appliedDefaults ? from : null,
				defaultTo: appliedDefaults ? to : null,
				error: 'Failed to load attendance data'
			};
		}

		const data = await response.json();

		// Fetch branches if user is company_manager or logsmart_admin
		let branches: Array<{ id: string; name: string; address: string }> = [];
		if (user.role === 'company_manager' || user.role === 'logsmart_admin' || isHQStaff) {
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
			members = (membersData.members ?? []).map((m: UserResponse) => ({
				id: m.id,
				branch_id: m.branch_id
			}));
		}

		const d = {
			clockEvents: data.events ?? [],
			nextCursor: data.next_cursor ?? null,
			user,
			branches,
			userRole: user.role,
			isHQStaff,
			members,
			defaultFrom: appliedDefaults ? from : null,
			defaultTo: appliedDefaults ? to : null
		};

		return d;
	} catch (error) {
		console.error('Error fetching attendance data:', error);
		const defaultRange = getDefaultDateRange();
		return {
			clockEvents: [],
			user,
			branches: [],
			userRole: user.role,
			members: [],
			defaultFrom: defaultRange.from,
			defaultTo: defaultRange.to,
			error: 'Failed to load attendance data'
		};
	}
};
