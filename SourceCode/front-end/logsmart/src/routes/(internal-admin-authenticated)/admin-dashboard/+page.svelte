<script lang="ts">
	import type { PageData } from './$types';

	let { data } = $props<{ data: PageData }>();
	let activeTab = $state<'database'>('database');

	// Database health monitoring data
	const dbHealth = $derived(data?.dbHealth ?? null);
	const tableSizes = $derived(data?.tableSizes ?? null);
	const slowQueries = $derived(data?.slowQueries ?? null);
	const indexUsage = $derived(data?.indexUsage ?? null);

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
		{/if}
	</div>
</div>
