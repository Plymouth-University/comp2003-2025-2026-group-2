import { json, redirect, type RequestEvent } from '@sveltejs/kit';
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
	} else {
		const token = cookies.get('ls-token');
		if (token) {
			headers.set('Authorization', `Bearer ${token}`);
		}
	}

	const originalUrl = new URL(request.url);
	const queryString = originalUrl.search;
	const url = `${API_URL}/${path}${queryString}`;

	let body = undefined;
	if (request.method !== 'GET' && request.method !== 'HEAD') {
		body = await request.text();
	}

	try {
		const response = await fetch(url, {
			method: request.method,
			headers,
			body,
			redirect: 'manual'
		});

		if (
			response.status === 301 ||
			response.status === 302 ||
			response.status === 303 ||
			response.status === 307 ||
			response.status === 308
		) {
			const location = response.headers.get('location');
			console.log('Redirect detected:', response.status, 'Location:', location);
			if (location) {
				throw redirect(303, location);
			} else {
				console.error('Redirect without location header:', response.status);
				return json({ error: 'Invalid redirect response' }, { status: 500 });
			}
		}

		const setCookieHeader = response.headers.get('set-cookie');
		if (setCookieHeader) {
			const tokenMatch = setCookieHeader.match(/ls-token=([^;]+)/);
			if (tokenMatch) {
				const token = tokenMatch[1];
				cookies.set('ls-token', token, {
					path: '/',
					httpOnly: true,
					secure: true,
					sameSite: 'lax',
					maxAge: 60 * 60 * 24 * 7
				});
			}
			const linkPendingMatch = setCookieHeader.match(/oauth_link_pending=([^;]+)/);
			if (linkPendingMatch) {
				const linkToken = linkPendingMatch[1];
				cookies.set('oauth_link_pending', linkToken, {
					path: '/',
					httpOnly: false,
					secure: true,
					sameSite: 'lax',
					maxAge: 300
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
	} catch (error: any) {
		if (error?.status && error?.location) {
			throw error;
		}
		console.error('Proxy error:', error);
		return json({ error: 'Network error' }, { status: 500 });
	}
}

export const GET = proxyRequest;
export const POST = proxyRequest;
export const PUT = proxyRequest;
export const DELETE = proxyRequest;
export const PATCH = proxyRequest;
