import { redirect } from '@sveltejs/kit';
import type { PageServerLoad } from './$types';

export const load: PageServerLoad = async ({ parent }) => {
	const { user } = await parent();

	// Deny access to staff and readonly HQ (readonly HQ can view but not edit templates)
	const isReadonlyHQ = user?.role === 'staff' && !user?.branch_id;
	if (user?.role === 'staff' || isReadonlyHQ) {
		throw redirect(303, '/logs-list');
	}
};
