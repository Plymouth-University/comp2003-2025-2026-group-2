import { redirect } from '@sveltejs/kit';
import type { PageServerLoad } from './$types';

export const load: PageServerLoad = async ({ parent, fetch, cookies }) => {
	const { user } = await parent();

	const isHQStaff = user?.role === 'staff' && !user?.branch_id;

	if (user?.role !== 'company_manager' && user?.role !== 'logsmart_admin' && !isHQStaff) {
		throw redirect(303, '/dashboard');
	}

	const token = cookies.get('ls-token');
	if (!token) {
		throw redirect(303, '/login');
	}

	try {
		const response = await fetch('/api/auth/company/branches');
		if (!response.ok) {
			return {
				branches: [],
				error: 'Failed to fetch branches'
			};
		}

		const data = await response.json();
		return {
			branches: data.branches || [],
			user,
			isHQStaff
		};
	} catch (error) {
		console.error('Error fetching branches:', error);
		return {
			branches: [],
			error: 'An unexpected error occurred'
		};
	}
};
