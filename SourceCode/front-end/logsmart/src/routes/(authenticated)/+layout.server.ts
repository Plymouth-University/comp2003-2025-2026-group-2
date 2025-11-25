import { redirect, type RequestEvent } from '@sveltejs/kit';

export const load = async ({ cookies, fetch }: RequestEvent) => {
	const token = cookies.get('ls-token');

	if (!token) {
		throw redirect(303, '/login');
	}

	try {
		const response = await fetch('/api/auth/verify', {
			method: 'POST',
			headers: {
				'Content-Type': 'application/json'
			},
			body: JSON.stringify({ token })
		});

		if (!response.ok) {
			cookies.delete('ls-token', { path: '/' });
			throw redirect(303, '/login');
		}

		const data = await response.json();

		return {
			user: {
				email: data.email
			}
		};
	} catch (error) {
		cookies.delete('ls-token', { path: '/' });
		throw redirect(303, '/login');
	}
};
