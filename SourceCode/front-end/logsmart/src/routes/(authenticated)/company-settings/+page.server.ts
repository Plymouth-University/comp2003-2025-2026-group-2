import type { PageServerLoad, PageServerLoadEvent } from './$types';

interface CompanyData {
	user: Record<string, unknown> | null;
	company: Record<string, unknown> | null;
}

export const load: PageServerLoad = async (event: PageServerLoadEvent): Promise<CompanyData> => {
	const token = event.cookies.get('ls-token');

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

		const userRes = await event.fetch('/api/auth/me', {
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

		const companyRes = await event.fetch(`/api/companies/${user.company_id}`, {
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
