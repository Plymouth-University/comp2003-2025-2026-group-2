import type { components } from '$lib/api-types';

type GetTemplateResponse = components['schemas']['GetTemplateResponse'];
type LogEntryResponse = components['schemas']['LogEntryResponse'];

export const load = async ({ url, fetch, cookies }: any) => {
	const token = cookies.get('ls-token');
	const templateName = url.searchParams.get('template');
	const entryId = url.searchParams.get('entry');
	const mode = url.searchParams.get('mode') || 'view';

	if (!token) {
		return {
			error: 'No authentication token',
			template: null,
			entry: null
		};
	}

	try {
		if (entryId) {
			const response = await fetch(`/api/logs/entries/${entryId}`, {
				headers: {
					Authorization: `Bearer ${token}`
				}
			});

			if (!response.ok) {
				const errorText = await response.text();
				console.error('Entry fetch error:', errorText);
				return {
					error: `Failed to fetch log entry: ${errorText}`,
					template: null,
					entry: null
				};
			}

			const entry: LogEntryResponse = await response.json();

			const templateResponse = await fetch(
				`/api/logs/templates?template_name=${encodeURIComponent(entry.template_name)}`,
				{
					headers: {
						Authorization: `Bearer ${token}`
					}
				}
			);

			if (!templateResponse.ok) {
				const errorText = await templateResponse.text();
				console.error('Template fetch error for entry:', errorText);
				return {
					error: `Failed to fetch template: ${errorText}`,
					template: null,
					entry
				};
			}

			const template: GetTemplateResponse = await templateResponse.json();

			return {
				entry,
				template,
				error: null,
				mode: mode === 'edit' ? 'edit' : 'view'
			};
		} else if (templateName) {
			const response = await fetch(
				`/api/logs/templates?template_name=${encodeURIComponent(templateName)}`,
				{
					headers: {
						Authorization: `Bearer ${token}`
					}
				}
			);

			if (!response.ok) {
				const errorText = await response.text();
				console.error('Template fetch error:', errorText);
				return {
					error: `Failed to fetch template: ${errorText}`,
					template: null,
					entry: null
				};
			}

			const template: GetTemplateResponse = await response.json();

			const createResponse = await fetch('/api/logs/entries', {
				method: 'POST',
				headers: {
					Authorization: `Bearer ${token}`,
					'Content-Type': 'application/json'
				},
				body: JSON.stringify({
					template_name: templateName
				})
			});

			if (!createResponse.ok) {
				const errorText = await createResponse.text();
				console.error('Create entry error:', errorText);
				return {
					error: `Failed to create log entry: ${errorText}`,
					template,
					entry: null,
					mode: 'create'
				};
			}

			const createResult = await createResponse.json();

			return {
				template,
				entry: null,
				entryId: createResult.id,
				error: null,
				mode: 'create'
			};
		}

		return {
			error: 'No template or entry specified',
			template: null,
			entry: null
		};
	} catch (error) {
		console.error('Error loading log template:', error);
		return {
			error: 'Failed to load data',
			template: null,
			entry: null
		};
	}
};
