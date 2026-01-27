export const load = async ({ parent, fetch, cookies }: any) => {
	const { user } = await parent();
	const token = cookies.get('ls-token');

	if (!token) {
		return {
			user,
			companies: [],
			dbHealth: null,
			tableSizes: null,
			slowQueries: null,
			indexUsage: null,
			error: 'No authentication token'
		};
	}

	const headers = {
		Authorization: `Bearer ${token}`,
		'Cache-Control': 'no-cache'
	};

	try {
		// Fetch all health monitoring data in parallel
		const [
			companiesResponse,
			dbHealthResponse,
			tableSizesResponse,
			slowQueriesResponse,
			indexUsageResponse
		] = await Promise.all([
			fetch('/api/admin/companies', { headers }),
			fetch('/api/health/database', { headers }),
			fetch('/api/health/table-sizes', { headers }),
			fetch('/api/health/slow-queries?limit=20', { headers }),
			fetch('/api/health/index-usage', { headers })
		]);

		// Debug logging for health endpoints
		console.log('DB Health Response:', dbHealthResponse.status, dbHealthResponse.statusText);
		console.log('Table Sizes Response:', tableSizesResponse.status, tableSizesResponse.statusText);
		console.log(
			'Slow Queries Response:',
			slowQueriesResponse.status,
			slowQueriesResponse.statusText
		);
		console.log('Index Usage Response:', indexUsageResponse.status, indexUsageResponse.statusText);

		const companies = companiesResponse.ok ? await companiesResponse.json() : { companies: [] };

		let dbHealth = null;
		if (dbHealthResponse.ok) {
			dbHealth = await dbHealthResponse.json();
		} else {
			const errorText = await dbHealthResponse.text();
			console.log('DB Health Error Body:', errorText);
		}

		let tableSizes = null;
		if (tableSizesResponse.ok) {
			tableSizes = await tableSizesResponse.json();
		} else {
			const errorText = await tableSizesResponse.text();
			console.log('Table Sizes Error Body:', errorText);
		}

		const slowQueries = slowQueriesResponse.ok ? await slowQueriesResponse.json() : null;
		const indexUsage = indexUsageResponse.ok ? await indexUsageResponse.json() : null;

		// Temporary mock company for testing if no companies exist
		const mockCompanies = [
			{
				id: 'temp-company-001',
				name: 'Test Company Ltd',
				created_at: new Date(Date.now() - 1000 * 60 * 60 * 24 * 30).toISOString()
			},
			{
				id: 'temp-company-002',
				name: 'Demo Industries Inc',
				created_at: new Date(Date.now() - 1000 * 60 * 60 * 24 * 15).toISOString()
			}
		];

		return {
			user,
			companies:
				companies.companies && companies.companies.length > 0 ? companies.companies : mockCompanies,
			dbHealth,
			tableSizes,
			slowQueries,
			indexUsage,
			error: null
		};
	} catch (error) {
		console.error('Error fetching admin data:', error);
		return {
			user,
			companies: [],
			dbHealth: null,
			tableSizes: null,
			slowQueries: null,
			indexUsage: null,
			error: 'Failed to fetch admin data'
		};
	}
};
