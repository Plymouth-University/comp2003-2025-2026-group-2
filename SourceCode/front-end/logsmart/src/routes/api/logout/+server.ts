import { redirect, type RequestEvent } from '@sveltejs/kit';

export const POST = async ({ cookies }: RequestEvent) => {
	cookies.delete('ls-token', { path: '/' });
	throw redirect(303, '/login');
};
