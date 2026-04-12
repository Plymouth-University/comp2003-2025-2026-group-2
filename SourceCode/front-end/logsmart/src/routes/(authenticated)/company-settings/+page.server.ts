import type { PageServerLoad, PageServerLoadEvent } from './$types';
import type { components } from '$lib/api-types';

type UserResponse = components['schemas']['UserResponse'];
type CompanyResponse = components['schemas']['CompanyResponse'];

interface PageData {
	user: UserResponse | null;
	company: CompanyResponse | null;
}

export const load: PageServerLoad = async (event: PageServerLoadEvent): Promise<PageData> => {
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

		const user = (await userRes.json()) as UserResponse;

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

		const company = (await companyRes.json()) as CompanyResponse;

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
