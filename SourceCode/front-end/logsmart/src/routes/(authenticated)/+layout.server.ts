import { redirect, type RequestEvent } from '@sveltejs/kit';

export const load = async ({ cookies, fetch }: RequestEvent) => {
	const token = cookies.get('ls-token');

	if (!token) {
		throw redirect(303, '/login');
	}

	try {
		const response = await fetch('/api/auth/me', {
			method: 'GET',
			headers: {
				Authorization: `Bearer ${token}`
			}
		});

		if (!response.ok) {
			cookies.delete('ls-token', { path: '/' });
			throw redirect(303, '/login');
		}

		const userData = await response.json();

		return {
			user: userData
		};
	} catch (error) {
		cookies.delete('ls-token', { path: '/' });
		throw redirect(303, '/login');
	}
};
