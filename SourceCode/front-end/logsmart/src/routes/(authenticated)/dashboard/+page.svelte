<script lang="ts">
	import type { PageData } from './$types';

	let { data } = $props<{ data: PageData }>();

	const todaysLogs = $derived(data?.todaysLogs ?? []);

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
		let role = '';
		switch (data.user.role) {
			case 'logsmart_admin':
				role = 'LogSmart Internal Administrator';
				break;
			case 'member':
				role = 'Member';
				break;
			case 'admin':
				role = 'Company Admin';
				break;
			default:
				role = 'Member';
				break;
		}

		return {
			name: fullName || 'User',
			email: data.user.email || '',
			company: data.user.company_name || 'N/A',
			role: role,
			initials: initials || '?'
		};
	});
</script>

<svelte:head>
	<title>Dashboard</title>
</svelte:head>
<div class="h-full w-full" style="background-color: var(--bg-secondary);">
	<!-- Main Content -->
	<div class="mx-auto max-w-7xl px-6 py-8">
		<!-- Header with User Profile -->
		<div class="mb-8 flex flex-wrap items-start justify-between">
			<h1 class="mb-8 text-3xl font-bold" style="color: var(--text-primary);">
				Dashboard Overview
			</h1>

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

		<!-- Today's Logs Section -->
		<div class="mb-8">
			<div class="inline-block border-2" style="border-color: var(--border-primary);">
				<div
					class="border-b-2 px-6 py-4"
					style="border-color: var(--border-primary); background-color: var(--bg-primary);"
				>
					<h2 class="text-xl font-bold" style="color: var(--text-primary);">Today's Logs</h2>
				</div>
				<div
					class="min-h-[280px] min-w-[380px] px-6 py-6"
					style="background-color: var(--bg-primary);"
				>
					{#if todaysLogs.length === 0}
						<div style="color: var(--text-secondary);">No logs due today</div>
					{:else}
						<ul class="space-y-2">
							{#each todaysLogs as log}
								<li style="color: var(--text-primary);">
									- {log.template_name}
									{log.period ? `(${log.period})` : ''}
								</li>
							{/each}
						</ul>
					{/if}
				</div>
			</div>
		</div>

		<!-- Quick Actions Section -->
		<div>
			<div class="inline-block border-2" style="border-color: var(--border-primary);">
				<div
					class="border-b-2 px-6 py-4"
					style="border-color: var(--border-primary); background-color: var(--bg-primary);"
				>
					<h2 class="text-xl font-bold" style="color: var(--text-primary);">Quick Actions</h2>
				</div>
				<div class="min-w-[380px] px-6 py-6" style="background-color: var(--bg-primary);">
					<div class="flex flex-col items-start gap-4">
						<button
							class="border-2 px-6 py-2 font-medium hover:opacity-80"
							style="border-color: var(--border-primary); color: var(--text-primary); background-color: var(--bg-primary);"
						>
							+ Create New Log
						</button>
						<button
							class="border-2 px-6 py-2 font-medium hover:opacity-80"
							style="border-color: var(--border-primary); color: var(--text-primary); background-color: var(--bg-primary);"
						>
							View Reports
						</button>
					</div>
				</div>
			</div>
		</div>
	</div>
</div>
