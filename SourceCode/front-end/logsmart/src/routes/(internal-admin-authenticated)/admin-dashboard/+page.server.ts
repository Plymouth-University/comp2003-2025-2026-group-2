export const load = async ({ parent, fetch, cookies }: any) => {
	const { user } = await parent();
	const token = cookies.get('ls-token');

	if (!token) {
		return {
			user,
			companies: [],
			error: 'No authentication token'
		};
	}

	try {
		// Fetch all companies (this endpoint needs to be created on the backend)
		const companiesResponse = await fetch('/api/admin/companies', {
			headers: {
				Authorization: `Bearer ${token}`,
				'Cache-Control': 'no-cache'
			}
		});

		const companies = companiesResponse.ok ? await companiesResponse.json() : { companies: [] };

		// Fetch metrics (recent logins/signups) - endpoint to be created
		const metricsResponse = await fetch('/api/admin/metrics', {
			headers: {
				Authorization: `Bearer ${token}`,
				'Cache-Control': 'no-cache'
			}
		});

		const metrics = metricsResponse.ok
			? await metricsResponse.json()
			: { recent_logins: [], recent_signups: [] };

		// Temporary mock data for testing
		const mockMetrics = {
			total_accounts: 45,
			logins_24h: 23,
			recent_logs: [
				{
					id: '1',
					template_name: 'Daily Safety Inspection',
					company_name: 'Tech Solutions Ltd',
					created_at: new Date(Date.now() - 1000 * 60 * 30).toISOString()
				},
				{
					id: '2',
					template_name: 'Maintenance Log',
					company_name: 'Global Industries Inc',
					created_at: new Date(Date.now() - 1000 * 60 * 60 * 2).toISOString()
				},
				{
					id: '3',
					template_name: 'Temperature Check',
					company_name: 'Smart Logistics Co',
					created_at: new Date(Date.now() - 1000 * 60 * 60 * 5).toISOString()
				}
			]
		};

		return {
			user,
			companies: companies.companies || [],
			metrics: metrics.total_accounts ? metrics : mockMetrics,
			error: null
		};
	} catch (error) {
		console.error('Error fetching admin data:', error);
		return {
			user,
			companies: [],
			metrics: { total_accounts: 0, logins_24h: 0, recent_logs: [] },
			error: 'Failed to fetch admin data'
		};
	}
};
