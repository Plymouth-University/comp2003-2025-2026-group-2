<script lang="ts">
	import type { PageData } from './$types';

	let { data } = $props<{ data: PageData }>();
	type AdminTab = 'database' | 'security';
	type SecurityLogRow = {
		id: string;
		event_type: string;
		user_id: string | null;
		email: string | null;
		ip_address: string | null;
		user_agent: string | null;
		actor_role: string | null;
		company_id: string | null;
		target_user_id: string | null;
		target_email: string | null;
		request_path: string | null;
		request_method: string | null;
		details: string | null;
		success: boolean;
		created_at: string;
	};

	let activeTab = $state<AdminTab>('database');

	// Database health monitoring data
	const dbHealth = $derived(data?.dbHealth ?? null);
	const tableSizes = $derived(data?.tableSizes ?? null);
	const slowQueries = $derived(data?.slowQueries ?? null);
	const indexUsage = $derived(data?.indexUsage ?? null);

	let securityLogs = $state<SecurityLogRow[]>([]);
	let securityNextCursor = $state<string | null>(null);
	let securityCurrentCursor = $state<string | null>(null);
	let securityCursorHistory = $state<string[]>([]);
	let securityLoading = $state(false);
	let securityLoaded = $state(false);
	let securityError = $state<string | null>(null);
	let exportingVisible = $state(false);
	let exportingAll = $state(false);

	let filterEventType = $state('');
	let filterUserId = $state('');
	let filterEmail = $state('');
	let filterIpAddress = $state('');
	let filterUserAgent = $state('');
	let filterActorRole = $state('');
	let filterCompanyId = $state('');
	let filterTargetUserId = $state('');
	let filterTargetEmail = $state('');
	let filterRequestPath = $state('');
	let filterRequestMethod = $state('');
	let filterDetails = $state('');
	let filterSuccess = $state('');
	let filterCreatedFrom = $state('');
	let filterCreatedTo = $state('');

	function appendDateFilter(params: URLSearchParams, key: string, value: string) {
		if (!value) {
			return;
		}
		const parsed = new Date(value);
		if (!Number.isNaN(parsed.getTime())) {
			params.set(key, parsed.toISOString());
		}
	}

	// Get user data from server load
	const user = $derived(() => {
		if (!data.user) {
			return {
				name: 'Loading...',
				email: '',
				company: '',
				role: '',
				initials: '?'
			};
		}

		const firstName = data.user.first_name || '';
		const lastName = data.user.last_name || '';
		const fullName = `${firstName} ${lastName}`.trim();
		const initials = (firstName.charAt(0) + lastName.charAt(0)).toUpperCase();

		return {
			name: fullName || 'User',
			email: data.user.email || '',
			company: data.user.company_name || 'N/A',
			role: data.user.role || 'Internal Admin',
			initials: initials || '?'
		};
	});

	$effect(() => {
		if (activeTab === 'security' && !securityLoaded && !securityLoading) {
			void fetchSecurityLogs({ reset: true });
		}
	});

	function appendFilter(params: URLSearchParams, key: string, value: string) {
		const trimmed = value.trim();
		if (trimmed) {
			params.set(key, trimmed);
		}
	}

	function createSecurityLogsSearchParams(cursor: string | null): URLSearchParams {
		const params = new URLSearchParams();
		params.set('limit', '15');
		if (cursor) {
			params.set('cursor', cursor);
		}

		appendFilter(params, 'event_type', filterEventType);
		appendFilter(params, 'user_id', filterUserId);
		appendFilter(params, 'email', filterEmail);
		appendFilter(params, 'ip_address', filterIpAddress);
		appendFilter(params, 'user_agent', filterUserAgent);
		appendFilter(params, 'actor_role', filterActorRole);
		appendFilter(params, 'company_id', filterCompanyId);
		appendFilter(params, 'target_user_id', filterTargetUserId);
		appendFilter(params, 'target_email', filterTargetEmail);
		appendFilter(params, 'request_path', filterRequestPath);
		appendFilter(params, 'request_method', filterRequestMethod);
		appendFilter(params, 'details', filterDetails);
		if (filterSuccess === 'true' || filterSuccess === 'false') {
			params.set('success', filterSuccess);
		}
		appendDateFilter(params, 'created_from', filterCreatedFrom);
		appendDateFilter(params, 'created_to', filterCreatedTo);

		return params;
	}

	async function fetchSecurityLogs({
		reset = false,
		cursor = null
	}: { reset?: boolean; cursor?: string | null } = {}) {
		securityLoading = true;
		securityError = null;

		if (reset) {
			securityCursorHistory = [];
			securityCurrentCursor = null;
		}

		const useCursor = reset ? null : cursor;
		const params = createSecurityLogsSearchParams(useCursor ?? null);

		try {
			const response = await fetch(`/api/security/logs?${params.toString()}`);
			if (!response.ok) {
				const errorBody = await response.json().catch(() => ({}));
				throw new Error(errorBody.error || 'Failed to fetch security logs');
			}

			const payload = await response.json();
			securityLogs = payload.logs ?? [];
			securityNextCursor = payload.next_cursor ?? null;
			securityCurrentCursor = useCursor ?? null;
			securityLoaded = true;
		} catch (error) {
			securityError = error instanceof Error ? error.message : 'Failed to fetch security logs';
		} finally {
			securityLoading = false;
		}
	}

	async function goToNextPage() {
		if (!securityNextCursor || securityLoading) {
			return;
		}
		securityCursorHistory = [...securityCursorHistory, securityCurrentCursor ?? ''];
		await fetchSecurityLogs({ cursor: securityNextCursor });
	}

	async function goToPreviousPage() {
		if (securityLoading || securityCursorHistory.length === 0) {
			return;
		}
		const previousCursor = securityCursorHistory[securityCursorHistory.length - 1] || null;
		securityCursorHistory = securityCursorHistory.slice(0, -1);
		await fetchSecurityLogs({ cursor: previousCursor });
	}

	async function applySecurityFilters() {
		await fetchSecurityLogs({ reset: true });
	}

	function csvEscape(value: string): string {
		return `"${value.replaceAll('"', '""')}"`;
	}

	function exportVisibleCsv() {
		if (securityLogs.length === 0 || exportingVisible) {
			return;
		}
		exportingVisible = true;
		try {
			const header = [
				'id',
				'event_type',
				'user_id',
				'email',
				'ip_address',
				'user_agent',
				'actor_role',
				'company_id',
				'target_user_id',
				'target_email',
				'request_path',
				'request_method',
				'details',
				'success',
				'created_at'
			];
			const rows = securityLogs.map((row) =>
				[
					row.id,
					row.event_type,
					row.user_id ?? '',
					row.email ?? '',
					row.ip_address ?? '',
					row.user_agent ?? '',
					row.actor_role ?? '',
					row.company_id ?? '',
					row.target_user_id ?? '',
					row.target_email ?? '',
					row.request_path ?? '',
					row.request_method ?? '',
					row.details ?? '',
					row.success ? 'true' : 'false',
					row.created_at
				]
					.map((cell) => csvEscape(String(cell)))
					.join(',')
			);
			const csv = `${header.join(',')}\n${rows.join('\n')}`;

			const blob = new Blob([csv], { type: 'text/csv;charset=utf-8' });
			const url = URL.createObjectURL(blob);
			const anchor = document.createElement('a');
			anchor.href = url;
			anchor.download = `security-logs-visible-${new Date().toISOString().replaceAll(':', '-')}.csv`;
			document.body.appendChild(anchor);
			anchor.click();
			anchor.remove();
			URL.revokeObjectURL(url);
		} finally {
			exportingVisible = false;
		}
	}

	function exportAllCsv() {
		if (exportingAll) {
			return;
		}
		exportingAll = true;
		try {
			const params = createSecurityLogsSearchParams(null);
			params.delete('limit');
			const anchor = document.createElement('a');
			anchor.href = `/api/security/logs/export?${params.toString()}`;
			anchor.rel = 'noopener';
			document.body.appendChild(anchor);
			anchor.click();
			anchor.remove();
		} finally {
			setTimeout(() => {
				exportingAll = false;
			}, 400);
		}
	}
</script>

<svelte:head>
	<title>Admin Dashboard</title>
</svelte:head>
<div class="h-full w-full" style="background-color: var(--bg-secondary);">
	<!-- Main Content -->
	<div class="mx-auto max-w-7xl px-6 py-8">
		<!-- Header with User Profile -->
		<div class="mb-8 flex flex-wrap items-start justify-between">
			<div class="flex-1">
				<h1 class="mb-4 text-3xl font-bold" style="color: var(--text-primary);">
					Internal Admin Dashboard
				</h1>
				<!-- Tab Navigation -->
				<div class="mb-6 flex gap-2">
					<button
						onclick={() => (activeTab = 'database')}
						class="rounded-t-lg px-6 py-3 font-semibold transition-colors"
						style={activeTab === 'database'
							? 'background-color: #3D7A82; color: white;'
							: 'background-color: var(--bg-primary); color: var(--text-secondary); border: 2px solid var(--border-primary);'}
					>
						Database Health
					</button>
					<button
						onclick={() => (activeTab = 'security')}
						class="rounded-t-lg px-6 py-3 font-semibold transition-colors"
						style={activeTab === 'security'
							? 'background-color: #3D7A82; color: white;'
							: 'background-color: var(--bg-primary); color: var(--text-secondary); border: 2px solid var(--border-primary);'}
					>
						Security Log
					</button>
				</div>
			</div>

			<!-- User Profile Section -->
			<div class="inline-block border-2" style="border-color: var(--border-primary);">
				<div class="px-6 py-4" style="background-color: var(--bg-primary);">
					<div class="flex items-center gap-4">
						<!-- Profile Picture (Initials) -->
						<div
							class="flex h-16 w-16 items-center justify-center rounded-full text-xl font-bold text-white"
							style="background-color: #3D7A82;"
						>
							{user().initials}
						</div>
						<!-- User Info -->
						<div class="text-left">
							<div class="font-bold" style="color: var(--text-primary);">{user().name}</div>
							<div class="text-sm" style="color: var(--text-secondary);">{user().email}</div>
							<div class="text-sm" style="color: var(--text-secondary);">{user().company}</div>
							<div class="text-sm font-medium" style="color: var(--text-primary);">
								{user().role === 'logsmart_admin' ? 'Logsmart Internal Admin' : user().role}
							</div>
						</div>
					</div>
				</div>
			</div>
		</div>

		{#if data.error}
			<div
				class="mb-4 rounded border-2 p-4"
				style="background-color: #fee; border-color: #fcc; color: #c00;"
			>
				{data.error}
			</div>
		{/if}

		<!-- Database Health Tab -->
		{#if activeTab === 'database'}
			<!-- Database Health Overview -->
			{#if dbHealth}
				<div class="mb-8">
					<h2 class="mb-4 text-2xl font-bold" style="color: var(--text-primary);">
						Database Overview
					</h2>
					<div class="grid grid-cols-2 gap-6 md:grid-cols-4">
						<!-- Status -->
						<div class="border-2" style="border-color: var(--border-primary);">
							<div class="px-6 py-4" style="background-color: var(--bg-primary);">
								<div class="text-sm font-medium" style="color: var(--text-secondary);">Status</div>
								<div
									class="mt-2 text-2xl font-bold"
									style="color: {dbHealth.status === 'healthy' ? '#5cb85c' : '#d9534f'};"
								>
									{dbHealth.status?.toUpperCase() ?? 'UNKNOWN'}
								</div>
							</div>
						</div>
						<!-- Database Size -->
						<div class="border-2" style="border-color: var(--border-primary);">
							<div class="px-6 py-4" style="background-color: var(--bg-primary);">
								<div class="text-sm font-medium" style="color: var(--text-secondary);">
									Database Size
								</div>
								<div class="mt-2 text-2xl font-bold" style="color: var(--text-primary);">
									{dbHealth.metrics?.database_size_mb?.toFixed(2) ?? '0'} MB
								</div>
							</div>
						</div>
						<!-- Active Connections -->
						<div class="border-2" style="border-color: var(--border-primary);">
							<div class="px-6 py-4" style="background-color: var(--bg-primary);">
								<div class="text-sm font-medium" style="color: var(--text-secondary);">
									Active Connections
								</div>
								<div class="mt-2 text-2xl font-bold" style="color: var(--text-primary);">
									{dbHealth.metrics?.active_connections ?? 0} / {dbHealth.metrics
										?.max_connections ?? 0}
								</div>
							</div>
						</div>
						<!-- Tables -->
						<div class="border-2" style="border-color: var(--border-primary);">
							<div class="px-6 py-4" style="background-color: var(--bg-primary);">
								<div class="text-sm font-medium" style="color: var(--text-secondary);">
									Tables / Indexes
								</div>
								<div class="mt-2 text-2xl font-bold" style="color: var(--text-primary);">
									{dbHealth.metrics?.table_count ?? 0} / {dbHealth.metrics?.index_count ?? 0}
								</div>
							</div>
						</div>
					</div>
					<!-- Connection Stats -->
					<div class="mt-4 grid grid-cols-3 gap-6">
						<div class="border-2" style="border-color: var(--border-primary);">
							<div class="px-6 py-4" style="background-color: var(--bg-primary);">
								<div class="text-sm font-medium" style="color: var(--text-secondary);">
									Total Connections
								</div>
								<div class="mt-2 text-xl font-bold" style="color: var(--text-primary);">
									{dbHealth.metrics?.total_connections ?? 0}
								</div>
							</div>
						</div>
						<div class="border-2" style="border-color: var(--border-primary);">
							<div class="px-6 py-4" style="background-color: var(--bg-primary);">
								<div class="text-sm font-medium" style="color: var(--text-secondary);">
									Idle Connections
								</div>
								<div class="mt-2 text-xl font-bold" style="color: var(--text-primary);">
									{dbHealth.metrics?.idle_connections ?? 0}
								</div>
							</div>
						</div>
						<div class="border-2" style="border-color: var(--border-primary);">
							<div class="px-6 py-4" style="background-color: var(--bg-primary);">
								<div class="text-sm font-medium" style="color: var(--text-secondary);">
									Max Connections
								</div>
								<div class="mt-2 text-xl font-bold" style="color: var(--text-primary);">
									{dbHealth.metrics?.max_connections ?? 0}
								</div>
							</div>
						</div>
					</div>
				</div>
			{:else}
				<div
					class="mb-8 rounded border-2 p-4"
					style="border-color: var(--border-primary); background-color: var(--bg-primary);"
				>
					<p style="color: var(--text-secondary);">
						Unable to load database health metrics. You may not have permission or the server is
						unavailable.
					</p>
				</div>
			{/if}

			<!-- Table Sizes -->
			{#if tableSizes?.tables && tableSizes.tables.length > 0}
				<div class="mb-8">
					<h2 class="mb-4 text-2xl font-bold" style="color: var(--text-primary);">Table Sizes</h2>
					<div
						class="overflow-hidden rounded border-2"
						style="border-color: var(--border-primary);"
					>
						<div class="overflow-x-auto">
							<table class="w-full" style="background-color: var(--bg-primary);">
								<thead>
									<tr style="background-color: var(--bg-secondary);">
										<th
											class="px-4 py-3 text-left text-sm font-semibold"
											style="color: var(--text-primary);">Table Name</th
										>
										<th
											class="px-4 py-3 text-right text-sm font-semibold"
											style="color: var(--text-primary);">Row Count</th
										>
										<th
											class="px-4 py-3 text-right text-sm font-semibold"
											style="color: var(--text-primary);">Table Size (MB)</th
										>
										<th
											class="px-4 py-3 text-right text-sm font-semibold"
											style="color: var(--text-primary);">Index Size (MB)</th
										>
										<th
											class="px-4 py-3 text-right text-sm font-semibold"
											style="color: var(--text-primary);">Total Size (MB)</th
										>
									</tr>
								</thead>
								<tbody>
									{#each tableSizes.tables as table (table.table_name)}
										<tr class="border-t" style="border-color: var(--border-primary);">
											<td class="px-4 py-3 text-sm font-medium" style="color: var(--text-primary);"
												>{table.table_name}</td
											>
											<td class="px-4 py-3 text-right text-sm" style="color: var(--text-secondary);"
												>{table.row_count?.toLocaleString() ?? '0'}</td
											>
											<td class="px-4 py-3 text-right text-sm" style="color: var(--text-secondary);"
												>{table.table_size_mb?.toFixed(3) ?? '0'}</td
											>
											<td class="px-4 py-3 text-right text-sm" style="color: var(--text-secondary);"
												>{table.index_size_mb?.toFixed(3) ?? '0'}</td
											>
											<td
												class="px-4 py-3 text-right text-sm font-medium"
												style="color: var(--text-primary);"
												>{table.total_size_mb?.toFixed(3) ?? '0'}</td
											>
										</tr>
									{/each}
								</tbody>
							</table>
						</div>
					</div>
				</div>
			{/if}

			<!-- Slow Queries -->
			{#if slowQueries?.queries && slowQueries.queries.length > 0}
				<div class="mb-8">
					<h2 class="mb-4 text-2xl font-bold" style="color: var(--text-primary);">Slow Queries</h2>
					<div
						class="overflow-hidden rounded border-2"
						style="border-color: var(--border-primary);"
					>
						<div class="overflow-x-auto">
							<table class="w-full" style="background-color: var(--bg-primary);">
								<thead>
									<tr style="background-color: var(--bg-secondary);">
										<th
											class="px-4 py-3 text-left text-sm font-semibold"
											style="color: var(--text-primary);">Query</th
										>
										<th
											class="px-4 py-3 text-right text-sm font-semibold"
											style="color: var(--text-primary);">Calls</th
										>
										<th
											class="px-4 py-3 text-right text-sm font-semibold"
											style="color: var(--text-primary);">Mean Time (ms)</th
										>
										<th
											class="px-4 py-3 text-right text-sm font-semibold"
											style="color: var(--text-primary);">Max Time (ms)</th
										>
										<th
											class="px-4 py-3 text-right text-sm font-semibold"
											style="color: var(--text-primary);">Total Time (ms)</th
										>
									</tr>
								</thead>
								<tbody>
									{#each slowQueries.queries as query (query.query)}
										<tr class="border-t" style="border-color: var(--border-primary);">
											<td class="max-w-md px-4 py-3 text-sm">
												<div
													class="truncate font-mono text-xs"
													style="color: var(--text-primary);"
													title={query.query}
												>
													{query.query}
												</div>
											</td>
											<td class="px-4 py-3 text-right text-sm" style="color: var(--text-secondary);"
												>{query.calls?.toLocaleString() ?? '0'}</td
											>
											<td class="px-4 py-3 text-right text-sm" style="color: var(--text-secondary);"
												>{query.mean_time_ms?.toFixed(2) ?? '0'}</td
											>
											<td class="px-4 py-3 text-right text-sm" style="color: #f59e0b;"
												>{query.max_time_ms?.toFixed(2) ?? '0'}</td
											>
											<td
												class="px-4 py-3 text-right text-sm font-medium"
												style="color: var(--text-primary);"
												>{query.total_time_ms?.toFixed(2) ?? '0'}</td
											>
										</tr>
									{/each}
								</tbody>
							</table>
						</div>
					</div>
				</div>
			{:else if slowQueries}
				<div
					class="mb-8 rounded border-2 p-4"
					style="border-color: var(--border-primary); background-color: var(--bg-primary);"
				>
					<h2 class="mb-2 text-xl font-bold" style="color: var(--text-primary);">Slow Queries</h2>
					<p style="color: #5cb85c;">No slow queries detected. Database is performing well!</p>
				</div>
			{/if}

			<!-- Index Usage -->
			{#if indexUsage}
				<div class="mb-8">
					<h2 class="mb-4 text-2xl font-bold" style="color: var(--text-primary);">Index Usage</h2>

					<!-- Unused Indexes Warning -->
					{#if indexUsage.unused_indexes && indexUsage.unused_indexes.length > 0}
						<div
							class="mb-4 rounded border-2 p-4"
							style="border-color: #f59e0b; background-color: rgba(245, 158, 11, 0.1);"
						>
							<h3 class="mb-2 font-semibold" style="color: #f59e0b;">
								⚠️ Unused Indexes ({indexUsage.unused_indexes.length})
							</h3>
							<p class="mb-2 text-sm" style="color: var(--text-secondary);">
								These indexes may be candidates for removal:
							</p>
							<ul class="list-inside list-disc text-sm" style="color: var(--text-primary);">
								{#each indexUsage.unused_indexes as idx, i (i)}
									<li class="font-mono">{idx}</li>
								{/each}
							</ul>
						</div>
					{/if}

					<!-- Index Stats Table -->
					{#if indexUsage.indexes && indexUsage.indexes.length > 0}
						<div
							class="overflow-hidden rounded border-2"
							style="border-color: var(--border-primary);"
						>
							<div class="overflow-x-auto">
								<table class="w-full" style="background-color: var(--bg-primary);">
									<thead>
										<tr style="background-color: var(--bg-secondary);">
											<th
												class="px-4 py-3 text-left text-sm font-semibold"
												style="color: var(--text-primary);">Index Name</th
											>
											<th
												class="px-4 py-3 text-left text-sm font-semibold"
												style="color: var(--text-primary);">Table</th
											>
											<th
												class="px-4 py-3 text-right text-sm font-semibold"
												style="color: var(--text-primary);">Scans</th
											>
											<th
												class="px-4 py-3 text-right text-sm font-semibold"
												style="color: var(--text-primary);">Rows Read</th
											>
											<th
												class="px-4 py-3 text-right text-sm font-semibold"
												style="color: var(--text-primary);">Rows Fetched</th
											>
										</tr>
									</thead>
									<tbody>
										{#each indexUsage.indexes as idx (idx.index_name)}
											<tr class="border-t" style="border-color: var(--border-primary);">
												<td class="px-4 py-3 font-mono text-sm" style="color: var(--text-primary);"
													>{idx.index_name}</td
												>
												<td class="px-4 py-3 text-sm" style="color: var(--text-secondary);"
													>{idx.table_name}</td
												>
												<td
													class="px-4 py-3 text-right text-sm"
													style="color: {idx.index_scans > 0 ? '#5cb85c' : '#d9534f'};"
													>{idx.index_scans?.toLocaleString() ?? '0'}</td
												>
												<td
													class="px-4 py-3 text-right text-sm"
													style="color: var(--text-secondary);"
													>{idx.rows_read?.toLocaleString() ?? '0'}</td
												>
												<td
													class="px-4 py-3 text-right text-sm"
													style="color: var(--text-secondary);"
													>{idx.rows_fetched?.toLocaleString() ?? '0'}</td
												>
											</tr>
										{/each}
									</tbody>
								</table>
							</div>
						</div>
					{/if}
				</div>
			{/if}
		{:else if activeTab === 'security'}
			<div
				class="mb-6 border-2 p-4"
				style="border-color: var(--border-primary); background-color: var(--bg-primary);"
			>
				<div class="mb-4 flex flex-wrap items-end gap-3">
					<div>
						<label
							for="security-filter-event-type"
							class="mb-1 block text-sm font-medium"
							style="color: var(--text-secondary);">Event Type</label
						>
						<input
							id="security-filter-event-type"
							bind:value={filterEventType}
							class="border-2 px-3 py-2 text-sm"
							style="border-color: var(--border-primary); color: var(--text-primary);"
						/>
					</div>
					<div>
						<label
							for="security-filter-user-id"
							class="mb-1 block text-sm font-medium"
							style="color: var(--text-secondary);">User ID</label
						>
						<input
							id="security-filter-user-id"
							bind:value={filterUserId}
							class="border-2 px-3 py-2 text-sm"
							style="border-color: var(--border-primary); color: var(--text-primary);"
						/>
					</div>
					<div>
						<label
							for="security-filter-email"
							class="mb-1 block text-sm font-medium"
							style="color: var(--text-secondary);">Email</label
						>
						<input
							id="security-filter-email"
							bind:value={filterEmail}
							class="border-2 px-3 py-2 text-sm"
							style="border-color: var(--border-primary); color: var(--text-primary);"
						/>
					</div>
					<div>
						<label
							for="security-filter-ip-address"
							class="mb-1 block text-sm font-medium"
							style="color: var(--text-secondary);">IP Address</label
						>
						<input
							id="security-filter-ip-address"
							bind:value={filterIpAddress}
							class="border-2 px-3 py-2 text-sm"
							style="border-color: var(--border-primary); color: var(--text-primary);"
						/>
					</div>
					<div>
						<label
							for="security-filter-user-agent"
							class="mb-1 block text-sm font-medium"
							style="color: var(--text-secondary);">User Agent</label
						>
						<input
							id="security-filter-user-agent"
							bind:value={filterUserAgent}
							class="border-2 px-3 py-2 text-sm"
							style="border-color: var(--border-primary); color: var(--text-primary);"
						/>
					</div>
					<div>
						<label
							for="security-filter-actor-role"
							class="mb-1 block text-sm font-medium"
							style="color: var(--text-secondary);">Actor Role</label
						>
						<input
							id="security-filter-actor-role"
							bind:value={filterActorRole}
							class="border-2 px-3 py-2 text-sm"
							style="border-color: var(--border-primary); color: var(--text-primary);"
						/>
					</div>
					<div>
						<label
							for="security-filter-company-id"
							class="mb-1 block text-sm font-medium"
							style="color: var(--text-secondary);">Company ID</label
						>
						<input
							id="security-filter-company-id"
							bind:value={filterCompanyId}
							class="border-2 px-3 py-2 text-sm"
							style="border-color: var(--border-primary); color: var(--text-primary);"
						/>
					</div>
					<div>
						<label
							for="security-filter-target-user-id"
							class="mb-1 block text-sm font-medium"
							style="color: var(--text-secondary);">Target User ID</label
						>
						<input
							id="security-filter-target-user-id"
							bind:value={filterTargetUserId}
							class="border-2 px-3 py-2 text-sm"
							style="border-color: var(--border-primary); color: var(--text-primary);"
						/>
					</div>
					<div>
						<label
							for="security-filter-target-email"
							class="mb-1 block text-sm font-medium"
							style="color: var(--text-secondary);">Target Email</label
						>
						<input
							id="security-filter-target-email"
							bind:value={filterTargetEmail}
							class="border-2 px-3 py-2 text-sm"
							style="border-color: var(--border-primary); color: var(--text-primary);"
						/>
					</div>
					<div>
						<label
							for="security-filter-request-path"
							class="mb-1 block text-sm font-medium"
							style="color: var(--text-secondary);">Request Path</label
						>
						<input
							id="security-filter-request-path"
							bind:value={filterRequestPath}
							class="border-2 px-3 py-2 text-sm"
							style="border-color: var(--border-primary); color: var(--text-primary);"
						/>
					</div>
					<div>
						<label
							for="security-filter-request-method"
							class="mb-1 block text-sm font-medium"
							style="color: var(--text-secondary);">Request Method</label
						>
						<input
							id="security-filter-request-method"
							bind:value={filterRequestMethod}
							class="border-2 px-3 py-2 text-sm"
							style="border-color: var(--border-primary); color: var(--text-primary);"
						/>
					</div>
					<div>
						<label
							for="security-filter-details"
							class="mb-1 block text-sm font-medium"
							style="color: var(--text-secondary);">Details</label
						>
						<input
							id="security-filter-details"
							bind:value={filterDetails}
							class="border-2 px-3 py-2 text-sm"
							style="border-color: var(--border-primary); color: var(--text-primary);"
						/>
					</div>
					<div>
						<label
							for="security-filter-success"
							class="mb-1 block text-sm font-medium"
							style="color: var(--text-secondary);">Success</label
						>
						<select
							id="security-filter-success"
							bind:value={filterSuccess}
							class="border-2 px-3 py-2 text-sm"
							style="border-color: var(--border-primary); color: var(--text-primary);"
						>
							<option value="">Any</option>
							<option value="true">True</option>
							<option value="false">False</option>
						</select>
					</div>
					<div>
						<label
							for="security-filter-created-from"
							class="mb-1 block text-sm font-medium"
							style="color: var(--text-secondary);">Created From</label
						>
						<input
							id="security-filter-created-from"
							bind:value={filterCreatedFrom}
							type="datetime-local"
							class="border-2 px-3 py-2 text-sm"
							style="border-color: var(--border-primary); color: var(--text-primary);"
						/>
					</div>
					<div>
						<label
							for="security-filter-created-to"
							class="mb-1 block text-sm font-medium"
							style="color: var(--text-secondary);">Created To</label
						>
						<input
							id="security-filter-created-to"
							bind:value={filterCreatedTo}
							type="datetime-local"
							class="border-2 px-3 py-2 text-sm"
							style="border-color: var(--border-primary); color: var(--text-primary);"
						/>
					</div>
				</div>
				<div class="flex flex-wrap gap-3">
					<button
						onclick={applySecurityFilters}
						class="border-2 px-4 py-2 font-semibold"
						style="border-color: var(--border-primary); color: var(--text-primary);"
						disabled={securityLoading}>Apply Filters</button
					>
					<button
						onclick={() => void fetchSecurityLogs({ reset: true })}
						class="border-2 px-4 py-2 font-semibold"
						style="border-color: var(--border-primary); color: var(--text-primary);"
						disabled={securityLoading}>Refresh</button
					>
					<button
						onclick={exportVisibleCsv}
						class="border-2 px-4 py-2 font-semibold"
						style="border-color: var(--border-primary); color: var(--text-primary);"
						disabled={securityLoading || securityLogs.length === 0 || exportingVisible}
						>{exportingVisible ? 'Exporting...' : 'Export Visible CSV'}</button
					>
					<button
						onclick={exportAllCsv}
						class="border-2 px-4 py-2 font-semibold"
						style="border-color: var(--border-primary); color: var(--text-primary);"
						disabled={securityLoading || exportingAll}
						>{exportingAll ? 'Exporting...' : 'Export All CSV'}</button
					>
				</div>
			</div>

			{#if securityError}
				<div
					class="mb-4 rounded border-2 p-4"
					style="background-color: #fee; border-color: #fcc; color: #c00;"
				>
					{securityError}
				</div>
			{/if}

			<div class="overflow-hidden rounded border-2" style="border-color: var(--border-primary);">
				<div class="overflow-x-auto">
					<table class="w-full" style="background-color: var(--bg-primary);">
						<thead>
							<tr style="background-color: var(--bg-secondary);">
								<th
									class="px-4 py-3 text-left text-sm font-semibold"
									style="color: var(--text-primary);">Created At</th
								>
								<th
									class="px-4 py-3 text-left text-sm font-semibold"
									style="color: var(--text-primary);">Event Type</th
								>
								<th
									class="px-4 py-3 text-left text-sm font-semibold"
									style="color: var(--text-primary);">User ID</th
								>
								<th
									class="px-4 py-3 text-left text-sm font-semibold"
									style="color: var(--text-primary);">Email</th
								>
								<th
									class="px-4 py-3 text-left text-sm font-semibold"
									style="color: var(--text-primary);">IP Address</th
								>
								<th
									class="px-4 py-3 text-left text-sm font-semibold"
									style="color: var(--text-primary);">User Agent</th
								>
								<th
									class="px-4 py-3 text-left text-sm font-semibold"
									style="color: var(--text-primary);">Actor Role</th
								>
								<th
									class="px-4 py-3 text-left text-sm font-semibold"
									style="color: var(--text-primary);">Target Email</th
								>
								<th
									class="px-4 py-3 text-left text-sm font-semibold"
									style="color: var(--text-primary);">Request</th
								>
								<th
									class="px-4 py-3 text-left text-sm font-semibold"
									style="color: var(--text-primary);">Success</th
								>
								<th
									class="px-4 py-3 text-left text-sm font-semibold"
									style="color: var(--text-primary);">Details</th
								>
							</tr>
						</thead>
						<tbody>
							{#if securityLoading}
								<tr class="border-t" style="border-color: var(--border-primary);">
									<td
										colspan="12"
										class="px-4 py-6 text-center text-sm"
										style="color: var(--text-secondary);">Loading security events...</td
									>
								</tr>
							{:else if securityLogs.length === 0}
								<tr class="border-t" style="border-color: var(--border-primary);">
									<td
										colspan="12"
										class="px-4 py-6 text-center text-sm"
										style="color: var(--text-secondary);">No security events found.</td
									>
								</tr>
							{:else}
								{#each securityLogs as log (log.id)}
									<tr class="border-t" style="border-color: var(--border-primary);">
										<td class="px-4 py-3 text-sm" style="color: var(--text-secondary);"
											>{new Date(log.created_at).toLocaleString()}</td
										>
										<td class="px-4 py-3 text-sm font-medium" style="color: var(--text-primary);"
											>{log.event_type}</td
										>
										<td class="px-4 py-3 font-mono text-xs" style="color: var(--text-secondary);"
											>{log.user_id || '-'}</td
										>
										<td class="px-4 py-3 text-sm" style="color: var(--text-secondary);"
											>{log.email || '-'}</td
										>
										<td class="px-4 py-3 text-sm" style="color: var(--text-secondary);"
											>{log.ip_address || '-'}</td
										>
										<td class="max-w-sm px-4 py-3 text-sm" style="color: var(--text-secondary);">
											<div class="truncate" title={log.user_agent || ''}>
												{log.user_agent || '-'}
											</div>
										</td>
										<td class="px-4 py-3 text-sm" style="color: var(--text-secondary);"
											>{log.actor_role || '-'}</td
										>
										<td class="px-4 py-3 text-sm" style="color: var(--text-secondary);"
											>{log.target_email || '-'}</td
										>
										<td class="max-w-sm px-4 py-3 text-sm" style="color: var(--text-secondary);">
											<div
												class="truncate"
												title={`${log.request_method || ''} ${log.request_path || ''}`}
											>
												{log.request_method || '-'}
												{log.request_path || ''}
											</div>
										</td>
										<td
											class="px-4 py-3 text-sm font-semibold"
											style={`color: ${log.success ? '#5cb85c' : '#d9534f'};`}
											>{log.success ? 'true' : 'false'}</td
										>
										<td class="max-w-md px-4 py-3 text-sm" style="color: var(--text-secondary);">
											<div class="truncate" title={log.details || ''}>{log.details || '-'}</div>
										</td>
									</tr>
								{/each}
							{/if}
						</tbody>
					</table>
				</div>
			</div>

			<div class="mt-4 flex items-center gap-3">
				<button
					onclick={() => void goToPreviousPage()}
					class="border-2 px-4 py-2 text-sm font-semibold"
					style="border-color: var(--border-primary); color: var(--text-primary);"
					disabled={securityLoading || securityCursorHistory.length === 0}>Previous</button
				>
				<button
					onclick={() => void goToNextPage()}
					class="border-2 px-4 py-2 text-sm font-semibold"
					style="border-color: var(--border-primary); color: var(--text-primary);"
					disabled={securityLoading || !securityNextCursor}>Next</button
				>
				<span class="text-sm" style="color: var(--text-secondary);"
					>Showing up to 15 events per page</span
				>
			</div>
		{/if}
	</div>
</div>
