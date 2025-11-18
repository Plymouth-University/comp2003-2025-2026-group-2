import type { LayoutServerLoad } from './$types';

export const load: LayoutServerLoad = async ({ cookies }) => {
	const token = cookies.get('ls-token');
	
	return {
		isAuthenticated: !!token
	};
};
