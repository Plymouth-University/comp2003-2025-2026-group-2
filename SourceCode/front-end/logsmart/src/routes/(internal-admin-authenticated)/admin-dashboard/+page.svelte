<script lang="ts">
	import type { PageData } from './$types';

	let { data } = $props<{ data: PageData }>();

	// State management
	let selectedCompanyId = $state<string | null>(null);
	let companyUsers = $state<any[]>([]);
	let companyLogs = $state<any[]>([]);
	let loadingUsers = $state(false);
	let loadingLogs = $state(false);
	let error = $state<string | null>(null);

	const companies = $derived(data?.companies ?? []);
	const metrics = $derived(data?.metrics ?? { total_accounts: 0, logins_24h: 0, recent_logs: [] });

	const selectedCompany = $derived(companies.find((c: any) => c.id === selectedCompanyId) || null);

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

	// Fetch users for a specific company
	async function fetchCompanyUsers(companyId: string) {
		loadingUsers = true;
		error = null;
		try {
			const response = await fetch(`/api/admin/companies/${companyId}/users`, {
				headers: {
					'Cache-Control': 'no-cache'
				}
			});

			if (response.ok) {
				const result = await response.json();
				companyUsers = result.users || [];
			} else {
				error = 'Failed to fetch company users';
				companyUsers = [];
			}
		} catch (err) {
			console.error('Error fetching company users:', err);
			error = 'Error fetching company users';
			companyUsers = [];
		} finally {
			loadingUsers = false;
		}
	}

	// Fetch logs for a specific company
	async function fetchCompanyLogs(companyId: string) {
		loadingLogs = true;
		error = null;
		try {
			const response = await fetch(`/api/admin/companies/${companyId}/logs`, {
				headers: {
					'Cache-Control': 'no-cache'
				}
			});

			if (response.ok) {
				const result = await response.json();
				companyLogs = result.logs || [];
			} else {
				error = 'Failed to fetch company logs';
				companyLogs = [];
			}
		} catch (err) {
			console.error('Error fetching company logs:', err);
			error = 'Error fetching company logs';
			companyLogs = [];
		} finally {
			loadingLogs = false;
		}
	}

	// Handle company selection
	function selectCompany(companyId: string) {
		selectedCompanyId = companyId;
		fetchCompanyUsers(companyId);
		fetchCompanyLogs(companyId);
	}

	// Send password reset request for a user
	async function sendPasswordReset(userId: string, userEmail: string) {
		if (
			!confirm(
				`Are you sure you want to send a password reset request to ${userEmail}? This will send them an email with reset instructions.`
			)
		) {
			return;
		}

		try {
			const response = await fetch(`/api/admin/users/${userId}/reset-password`, {
				method: 'POST',
				headers: {
					'Content-Type': 'application/json'
				}
			});

			if (response.ok) {
				alert(`Password reset email sent to ${userEmail}`);
			} else {
				const errorData = await response.text();
				console.error('Password reset failed:', errorData);
				alert('Failed to send password reset email');
			}
		} catch (err) {
			console.error('Error sending password reset:', err);
			alert('Error sending password reset email');
		}
	}

	// Format date helper
	function formatDate(dateString: string): string {
		const date = new Date(dateString);
		return date.toLocaleString('en-GB', {
			day: '2-digit',
			month: '2-digit',
			year: 'numeric',
			hour: '2-digit',
			minute: '2-digit'
		});
	}

	// Format relative time (for metrics)
	function formatRelativeTime(dateString: string): string {
		const date = new Date(dateString);
		const now = new Date();
		const diffMs = now.getTime() - date.getTime();
		const diffMins = Math.floor(diffMs / 60000);
		const diffHours = Math.floor(diffMs / 3600000);
		const diffDays = Math.floor(diffMs / 86400000);

		if (diffMins < 1) return 'Just now';
		if (diffMins < 60) return `${diffMins} min${diffMins > 1 ? 's' : ''} ago`;
		if (diffHours < 24) return `${diffHours} hour${diffHours > 1 ? 's' : ''} ago`;
		if (diffDays < 7) return `${diffDays} day${diffDays > 1 ? 's' : ''} ago`;
		return formatDate(dateString);
	}

	// Get log statistics
	const logStats = $derived(() => {
		if (!companyLogs || companyLogs.length === 0) {
			return {
				total: 0,
				submitted: 0,
				draft: 0,
				pending: 0
			};
		}

		return {
			total: companyLogs.length,
			submitted: companyLogs.filter((log) => log.status === 'submitted').length,
			draft: companyLogs.filter((log) => log.status === 'draft').length,
			pending: companyLogs.filter((log) => log.status === 'pending').length
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
				<!-- Company Selector Dropdown -->
				<div class="max-w-md">
					<label
						for="company-select"
						class="mb-2 block font-medium"
						style="color: var(--text-secondary);"
					>
						Select Company
					</label>
					<select
						id="company-select"
						bind:value={selectedCompanyId}
						onchange={() => {
							if (selectedCompanyId) {
								fetchCompanyUsers(selectedCompanyId);
								fetchCompanyLogs(selectedCompanyId);
							}
						}}
						class="w-full rounded border-2 px-4 py-2"
						style="border-color: var(--border-primary); background-color: var(--bg-primary); color: var(--text-primary);"
					>
						<option value="">-- Select a company --</option>
						{#each companies as company}
							<option value={company.id}>{company.name}</option>
						{/each}
					</select>
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
								{user().role}
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

		<!-- Metrics and Recent Logs Container -->
		<div class="mb-8 max-w-3xl">
			<!-- Metrics Section -->
			<div class="mb-8 grid grid-cols-2 gap-6">
				<!-- Number of Accounts -->
				<div class="border-2" style="border-color: var(--border-primary);">
					<div
						class="border-b-2 px-6 py-4"
						style="border-color: var(--border-primary); background-color: var(--bg-primary);"
					>
						<h2 class="text-xl font-bold" style="color: var(--text-primary);">
							Number of Accounts
						</h2>
					</div>
					<div class="px-6 py-8" style="background-color: var(--bg-primary);">
						<div class="text-center text-5xl font-bold" style="color: var(--text-primary);">
							{metrics.total_accounts}
						</div>
					</div>
				</div>

				<!-- Number of Logins 24h -->
				<div class="border-2" style="border-color: var(--border-primary);">
					<div
						class="border-b-2 px-6 py-4"
						style="border-color: var(--border-primary); background-color: var(--bg-primary);"
					>
						<h2 class="text-xl font-bold" style="color: var(--text-primary);">
							Number of Logins 24h
						</h2>
					</div>
					<div class="px-6 py-8" style="background-color: var(--bg-primary);">
						<div class="text-center text-5xl font-bold" style="color: var(--text-primary);">
							{metrics.logins_24h}
						</div>
					</div>
				</div>
			</div>

			<!-- Recent Logs Created -->
			<div class="mb-8">
				<div class="border-2" style="border-color: var(--border-primary);">
					<div
						class="border-b-2 px-6 py-4"
						style="border-color: var(--border-primary); background-color: var(--bg-primary);"
					>
						<h2 class="text-xl font-bold" style="color: var(--text-primary);">
							Recent Logs Created
						</h2>
					</div>
					<div
						class="min-h-[200px] min-w-[600px] px-6 py-6"
						style="background-color: var(--bg-primary);"
					>
						{#if metrics.recent_logs && metrics.recent_logs.length > 0}
							<ol class="space-y-3">
								{#each metrics.recent_logs as log, index}
									<li style="color: var(--text-primary);">
										<div class="flex items-center gap-3">
											<span class="font-semibold">{index + 1})</span>
											<div class="flex-1">
												<div class="font-medium">{log.template_name}</div>
												<div class="text-sm" style="color: var(--text-secondary);">
													{log.company_name} - {formatRelativeTime(log.created_at)}
												</div>
											</div>
										</div>
									</li>
								{/each}
							</ol>
						{:else}
							<p style="color: var(--text-secondary);">No recent logs created</p>
						{/if}
					</div>
				</div>
			</div>
		</div>

		{#if selectedCompany}
			<!-- Company Users Section -->
			<div class="mb-8">
				<div class="inline-block border-2" style="border-color: var(--border-primary);">
					<div
						class="border-b-2 px-6 py-4"
						style="border-color: var(--border-primary); background-color: var(--bg-primary);"
					>
						<h2 class="text-xl font-bold" style="color: var(--text-primary);">
							Users - {selectedCompany.name}
						</h2>
					</div>
					<div
						class="min-h-[200px] min-w-[800px] px-6 py-6"
						style="background-color: var(--bg-primary);"
					>
						{#if error}
							<div style="color: #c00;">{error}</div>
						{:else if loadingUsers}
							<p style="color: var(--text-secondary);">Loading users...</p>
						{:else if companyUsers.length === 0}
							<p style="color: var(--text-secondary);">No users found for this company</p>
						{:else}
							<div class="space-y-2">
								{#each companyUsers as user}
									<div
										class="flex items-center justify-between rounded border-2 p-4"
										style="border-color: var(--border-primary); background-color: var(--bg-secondary);"
									>
										<div class="flex-1">
											<div class="font-semibold" style="color: var(--text-primary);">
												{user.first_name}
												{user.last_name}
											</div>
											<div class="text-sm" style="color: var(--text-secondary);">
												{user.email} | Role: {user.role}
											</div>
										</div>
										<button
											onclick={() => sendPasswordReset(user.id, user.email)}
											class="rounded px-4 py-2 font-medium text-white hover:opacity-80"
											style="background-color: #f59e0b;"
										>
											Reset Password
										</button>
									</div>
								{/each}
							</div>
						{/if}
					</div>
				</div>
			</div>

			<!-- Company Logs Section -->
			<div class="mb-8">
				<div class="inline-block border-2" style="border-color: var(--border-primary);">
					<div
						class="border-b-2 px-6 py-4"
						style="border-color: var(--border-primary); background-color: var(--bg-primary);"
					>
						<h2 class="text-xl font-bold" style="color: var(--text-primary);">
							Logs - {selectedCompany.name}
						</h2>
					</div>
					<div
						class="min-h-[300px] min-w-[800px] px-6 py-6"
						style="background-color: var(--bg-primary);"
					>
						{#if loadingLogs}
							<p style="color: var(--text-secondary);">Loading logs...</p>
						{:else}
							<!-- Log Statistics -->
							<div class="mb-6 grid grid-cols-4 gap-4">
								<div class="rounded border-2 p-4" style="border-color: var(--border-primary);">
									<div class="text-2xl font-bold" style="color: var(--text-primary);">
										{logStats().total}
									</div>
									<div class="text-sm" style="color: var(--text-secondary);">Total Logs</div>
								</div>
								<div class="rounded border-2 p-4" style="border-color: var(--border-primary);">
									<div class="text-2xl font-bold" style="color: #5cb85c;">
										{logStats().submitted}
									</div>
									<div class="text-sm" style="color: var(--text-secondary);">Submitted</div>
								</div>
								<div class="rounded border-2 p-4" style="border-color: var(--border-primary);">
									<div class="text-2xl font-bold" style="color: #f59e0b;">
										{logStats().draft}
									</div>
									<div class="text-sm" style="color: var(--text-secondary);">Draft</div>
								</div>
								<div class="rounded border-2 p-4" style="border-color: var(--border-primary);">
									<div class="text-2xl font-bold" style="color: #d9534f;">
										{logStats().pending}
									</div>
									<div class="text-sm" style="color: var(--text-secondary);">Pending</div>
								</div>
							</div>

							<!-- Logs List -->
							{#if companyLogs.length === 0}
								<p style="color: var(--text-secondary);">No logs found for this company</p>
							{:else}
								<div class="space-y-2">
									{#each companyLogs as log}
										<div
											class="flex items-center justify-between rounded border-2 p-4"
											style="border-color: var(--border-primary); background-color: var(--bg-secondary);"
										>
											<div class="flex-1">
												<div class="font-semibold" style="color: var(--text-primary);">
													{log.template_name}
													{log.period ? `(${log.period})` : ''}
												</div>
												<div class="text-sm" style="color: var(--text-secondary);">
													Status: <span
														class="font-medium"
														class:status-submitted={log.status === 'submitted'}
														class:status-draft={log.status === 'draft'}
														class:status-pending={log.status === 'pending'}
													>
														{log.status}
													</span>
													{#if log.created_at}
														| Created: {formatDate(log.created_at)}
													{/if}
													{#if log.submitted_at}
														| Submitted: {formatDate(log.submitted_at)}
													{/if}
												</div>
											</div>
										</div>
									{/each}
								</div>
							{/if}
						{/if}
					</div>
				</div>
			</div>
		{/if}
	</div>
</div>

<style>
	.status-submitted {
		color: #5cb85c;
	}

	.status-draft {
		color: #f59e0b;
	}

	.status-pending {
		color: #d9534f;
	}
</style>
