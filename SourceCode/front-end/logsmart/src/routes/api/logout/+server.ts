import { redirect, type RequestEvent } from '@sveltejs/kit';

export const POST = async ({ cookies, url }: RequestEvent) => {
	const opts: { path: string; domain?: string } = { path: '/' };
	if (url.hostname !== 'localhost' && url.hostname !== '127.0.0.1') {
		opts.domain = url.hostname;
	}
	cookies.delete('ls-token', opts);
	throw redirect(303, '/login');
};
