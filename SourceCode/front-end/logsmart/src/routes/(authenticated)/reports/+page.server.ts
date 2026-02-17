import { redirect } from '@sveltejs/kit';
import type { PageServerLoad } from './$types';

export const load: PageServerLoad = async ({ parent, fetch, cookies }) => {
	const { user } = await parent();

	// Allow readonly HQ users (staff with no branch) to access reports
	const isReadonlyHQ = user?.role === 'staff' && !user?.branch_id;
	const isCompanyManager = user?.role === 'company_manager';
	const isBranchManager = user?.role === 'branch_manager';

	// Branch managers can access reports, staff at branches cannot
	if (user?.role === 'staff' && !isReadonlyHQ) {
		throw redirect(303, '/logs-list');
	}

	// Fetch branches for company managers, HQ staff, and branch managers
	let branches: Array<{ id: string; name: string; address: string; created_at: string }> = [];
	let userBranchId = user?.branch_id;

	if (isCompanyManager || isReadonlyHQ) {
		const token = cookies.get('ls-token');
		if (token) {
			try {
				const response = await fetch('/api/auth/company/branches', {
					headers: {
						Authorization: `Bearer ${token}`,
						'Cache-Control': 'no-cache'
					}
				});
				if (response.ok) {
					const data = await response.json();
					branches = data.branches || [];
				}
			} catch (error) {
				console.error('Error fetching branches:', error);
			}
		}
	} else if (isBranchManager && userBranchId) {
		// Branch managers can only see their own branch
		branches = [{ id: userBranchId, name: 'Your Branch', address: '', created_at: '' }];
	}

	return {
		user,
		branches
	};
};
