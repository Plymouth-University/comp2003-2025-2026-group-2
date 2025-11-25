import { json, type RequestEvent } from '@sveltejs/kit';
import { PUBLIC_API_URL } from '$env/static/public';

const API_URL = PUBLIC_API_URL || 'https://api.logsmart.app';

async function proxyRequest(event: RequestEvent) {
	const { request, params, cookies } = event;
	let path = params.path || '';
	if (
		[
			'swagger-ui.css',
			'index.css',
			'swagger-ui-bundle.js',
			'swagger-ui-standalone-preset.js',
			'swagger-initializer.js'
		].includes(path)
	) {
		path = `swagger-ui/${path}`;
	}

	const headers = new Headers();
	headers.set('Content-Type', request.headers.get('Content-Type') || 'application/json');

	const authHeader = request.headers.get('Authorization');
	if (authHeader) {
		headers.set('Authorization', authHeader);
	}

	const url = `${API_URL}/${path}`;

	let body = undefined;
	if (request.method !== 'GET' && request.method !== 'HEAD') {
		body = await request.text();
	}

	try {
		const response = await fetch(url, {
			method: request.method,
			headers,
			body
		});

		const setCookieHeader = response.headers.get('set-cookie');
		if (setCookieHeader) {
			const tokenMatch = setCookieHeader.match(/ls-token=([^;]+)/);
			if (tokenMatch) {
				const token = tokenMatch[1];
				cookies.set('ls-token', token, {
					path: '/',
					httpOnly: true,
					secure: true,
					sameSite: 'strict',
					maxAge: 60 * 60 * 24 * 7
				});
			}
		}

		const contentType = response.headers.get('content-type');
		if (contentType?.includes('application/json')) {
			const data = await response.json();
			return json(data, { status: response.status });
		} else {
			const text = await response.text();
			return new Response(text, {
				status: response.status,
				headers: { 'content-type': contentType || 'text/plain' }
			});
		}
	} catch (error) {
		console.error('Proxy error:', error);
		return json({ error: 'Network error' }, { status: 500 });
	}
}

export const GET = proxyRequest;
export const POST = proxyRequest;
export const PUT = proxyRequest;
export const DELETE = proxyRequest;
export const PATCH = proxyRequest;
