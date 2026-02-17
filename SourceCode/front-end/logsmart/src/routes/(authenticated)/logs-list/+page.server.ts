import type { components } from '$lib/api-types';

type ListLogEntriesResponse = components['schemas']['ListLogEntriesResponse'];
type DueFormsResponse = components['schemas']['DueFormsResponse'];
type User = components['schemas']['UserResponse'];

export const load = async ({ parent, fetch, cookies }: any) => {
	const { user } = await parent();
	const token = cookies.get('ls-token');

	if (!token) {
		return {
			user: user as User,
			dueToday: [],
			pastLogs: [],
			error: 'No authentication token'
		};
	}

	try {
		// Check if user is readonly HQ (staff with no branch)
		const isReadonlyHQ = user?.role === 'staff' && !user?.branch_id;

		if (user?.role === 'staff' && !isReadonlyHQ) {
			const [dueTodayResponse, pastLogsResponse] = await Promise.all([
				fetch('/api/logs/entries/due', {
					headers: {
						Authorization: `Bearer ${token}`,
						'Cache-Control': 'no-cache'
					}
				}),
				fetch('/api/logs/entries', {
					headers: {
						Authorization: `Bearer ${token}`,
						'Cache-Control': 'no-cache'
					}
				})
			]);

			const dueToday: DueFormsResponse = dueTodayResponse.ok
				? await dueTodayResponse.json()
				: { forms: [] };
			const pastLogsData: ListLogEntriesResponse = pastLogsResponse.ok
				? await pastLogsResponse.json()
				: { entries: [] };

			return {
				user: user as User,
				dueToday: dueToday.forms || [],
				pastLogs: pastLogsData.entries || [],
				error: null
			};
		} else {
			const [dueTodayResponse, allLogsResponse] = await Promise.all([
				fetch('/api/logs/entries/due', {
					headers: {
						Authorization: `Bearer ${token}`,
						'Cache-Control': 'no-cache'
					}
				}),
				fetch('/api/logs/admin/entries', {
					headers: {
						Authorization: `Bearer ${token}`,
						'Cache-Control': 'no-cache'
					}
				})
			]);

			const dueToday: DueFormsResponse = dueTodayResponse.ok
				? await dueTodayResponse.json()
				: { forms: [] };
			const allLogsData: ListLogEntriesResponse = allLogsResponse.ok
				? await allLogsResponse.json()
				: { entries: [] };

			return {
				user: user as User,
				dueToday: dueToday.forms || [],
				allLogs: allLogsData.entries || [],
				error: null
			};
		}
	} catch (error) {
		console.error('Error fetching logs:', error);
		return {
			user: user as User,
			dueToday: [],
			pastLogs: [],
			allLogs: [],
			error: 'Failed to fetch logs'
		};
	}
};
