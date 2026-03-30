import type { PageServerLoad } from './$types';

export const load: PageServerLoad = async ({ fetch, cookies }) => {
	const token = cookies.get('ls-token');

	if (!token) {
		return {
			user: null,
			company: null
		};
	}

	try {
		const headers = {
			Authorization: `Bearer ${token}`
		};

		const userRes = await fetch('/api/auth/me', {
			method: 'GET',
			headers
		});

		if (!userRes.ok) {
			return {
				user: null,
				company: null
			};
		}

		const user = await userRes.json();

		if (!user.company_id) {
			return {
				user,
				company: null
			};
		}

		const companyRes = await fetch(`/api/companies/${user.company_id}`, {
			method: 'GET',
			headers
		});

		if (!companyRes.ok) {
			return {
				user,
				company: null
			};
		}

		const company = await companyRes.json();

		return {
			user,
			company
		};
	} catch (error) {
		console.error('Error fetching company data:', error);
		return {
			user: null,
			company: null
		};
	}
};
