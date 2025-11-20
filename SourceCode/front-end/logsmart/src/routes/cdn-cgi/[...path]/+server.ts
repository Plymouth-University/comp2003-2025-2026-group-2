import { PUBLIC_API_URL } from '$env/static/public';
import { json, type RequestEvent } from '@sveltejs/kit';

const API_URL = PUBLIC_API_URL || 'https://api.logsmart.app';

async function proxyRequest(event: RequestEvent) {
	const { request, params, cookies } = event;
	let path = params.path || '';
	if (path.endsWith("rocket-loader.min.js")) {
		path = `cdn-cgi/${path}`;
	}
	
	const headers = request.headers;
	
	const url = `${API_URL}/${path}`;

	try {
		const response = await fetch(url, {
			method: "GET",
			headers
		});

        return new Response(await response.text(), { headers: { 'content-type': 'application/javascript' , 'location': `/api/${path}`}, status: 301 });
	} catch (error) {
		console.error('Proxy error:', error);
		return json({ error: 'Network error' }, { status: 500 });
	}
}

export const GET = proxyRequest;
