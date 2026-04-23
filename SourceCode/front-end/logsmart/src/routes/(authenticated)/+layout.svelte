<script lang="ts">
	import { page } from '$app/state';
	import type { LayoutData } from './$types';

	let { data, children } = $props<{ data: LayoutData; children: import('svelte').Snippet }>();

	let mobileMenuOpen = $state(false);

	const currentPath = $derived(page.url.pathname);
	const isReadonlyHQ = $derived(data?.user?.role === 'staff' && !data?.user?.branch_id);
	const isBranchManager = $derived(data?.user?.role === 'branch_manager');
	const isCompanyManager = $derived(data?.user?.role === 'company_manager');
	const isAdmin = $derived(
		isCompanyManager || data?.user?.role === 'logsmart_admin' || isReadonlyHQ || isBranchManager
	);

	async function handleLogout() {
		await fetch('/api/logout', { method: 'POST' });
		window.location.href = '/login';
	}

	function closeMobileMenu() {
		mobileMenuOpen = false;
	}
</script>

<div class="flex h-full w-full flex-col bg-bg-secondary">
	<header class="shrink-0 border-b border-border-secondary bg-bg-primary shadow-sm">
		<div class="max-w-9xl mx-auto px-6 py-4 lg:px-30">
			<div class="flex items-center justify-between">
				<div class="flex items-center gap-8">
					<a
						href={data?.user?.role === 'logsmart_admin'
							? '/admin-dashboard'
							: isAdmin
								? '/dashboard'
								: '/logs-list'}
						class="text-2xl font-bold text-button-primary">LogSmart</a
					>
					<nav class="hidden flex-wrap items-center gap-4 md:flex md:gap-6">
						{#if data?.user?.role === 'logsmart_admin'}
							<a
								href="/admin-dashboard"
								class="text-text-secondary hover:opacity-80"
								class:font-bold={currentPath === '/admin-dashboard'}
								class:underline={currentPath === '/admin-dashboard'}
							>
								Admin Dashboard
							</a>
							<a
								href="/dashboard"
								class="text-text-secondary hover:opacity-80"
								class:font-bold={currentPath === '/dashboard'}
								class:underline={currentPath === '/dashboard'}
							>
								Dashboard
							</a>
						{:else if isAdmin}
							<a
								href="/dashboard"
								class="text-text-secondary hover:opacity-80"
								class:font-bold={currentPath === '/dashboard'}
								class:underline={currentPath === '/dashboard'}
							>
								Dashboard
							</a>
						{/if}
						<a
							href="/logs-list"
							class:font-bold={currentPath === '/logs-list'}
							class:underline={currentPath === '/logs-list'}
							class="text-text-secondary hover:opacity-80"
						>
							Logs
						</a>
						{#if isAdmin}
							<a
								href="/users-admin"
								class:font-bold={currentPath === '/users-admin'}
								class:underline={currentPath === '/users-admin'}
								class="text-text-secondary hover:opacity-80"
							>
								Users
							</a>
							<a
								href="/attendance-admin"
								class="text-text-secondary hover:opacity-80"
								class:font-bold={currentPath === '/attendance-admin'}
								class:underline={currentPath === '/attendance-admin'}
							>
								Attendance
							</a>
							<a
								href="/reports"
								class="text-text-secondary hover:opacity-80"
								class:font-bold={currentPath === '/reports'}
								class:underline={currentPath === '/reports'}
							>
								Reports
							</a>
							<a
								href="/templates-dashboard"
								class="text-text-secondary hover:opacity-80"
								class:font-bold={currentPath === '/templates-dashboard'}
								class:underline={currentPath === '/templates-dashboard'}
							>
								Templates Dashboard
							</a>
						{/if}
						{#if isCompanyManager || isReadonlyHQ || data?.user?.role === 'logsmart_admin'}
							<a
								href="/branches"
								class="text-text-secondary hover:opacity-80"
								class:font-bold={currentPath === '/branches'}
								class:underline={currentPath === '/branches'}
							>
								Branches
							</a>
						{/if}
					</nav>
				</div>
				<div class="flex items-center gap-4">
					<div class="hidden flex-wrap items-center gap-4 md:flex">
						{#if data?.user}
							<span class="text-sm text-text-secondary">{data.user.email}</span>
						{/if}
						<a
							href="/settings"
							class="rounded bg-bg-secondary px-4 py-2 text-text-primary hover:opacity-80"
						>
							Settings
						</a>
						{#if isCompanyManager || data?.user?.role === 'logsmart_admin'}
							<a
								href="/company-settings"
								class="rounded bg-bg-secondary px-4 py-2 text-text-primary hover:opacity-80"
							>
								Company Settings
							</a>
						{/if}
						<button
							onclick={handleLogout}
							class="transform cursor-pointer rounded bg-bg-secondary px-4 py-2 text-text-primary transition-all duration-150 hover:scale-105 hover:opacity-80"
						>
							Logout
						</button>
					</div>
					<button
						onclick={() => (mobileMenuOpen = !mobileMenuOpen)}
						class="p-2 text-text-secondary hover:opacity-80 md:hidden"
						aria-label="Toggle menu"
					>
						<svg class="h-6 w-6" fill="none" stroke="currentColor" viewBox="0 0 24 24">
							<path
								stroke-linecap="round"
								stroke-linejoin="round"
								stroke-width="2"
								d={mobileMenuOpen ? 'M6 18L18 6M6 6l12 12' : 'M4 6h16M4 12h16M4 18h16'}
							/>
						</svg>
					</button>
				</div>
			</div>
			{#if mobileMenuOpen}
				<nav class="mt-4 flex flex-col gap-3 border-t border-border-secondary pt-4 md:hidden">
					{#if data?.user?.role === 'logsmart_admin'}
						<a
							href="/admin-dashboard"
							class="block text-text-secondary hover:opacity-80"
							class:font-bold={currentPath === '/admin-dashboard'}
							onclick={closeMobileMenu}
						>
							Admin Dashboard
						</a>
						<a
							href="/dashboard"
							class="block text-text-secondary hover:opacity-80"
							class:font-bold={currentPath === '/dashboard'}
							onclick={closeMobileMenu}
						>
							Dashboard
						</a>
					{:else if isAdmin}
						<a
							href="/dashboard"
							class="block text-text-secondary hover:opacity-80"
							class:font-bold={currentPath === '/dashboard'}
							onclick={closeMobileMenu}
						>
							Dashboard
						</a>
					{/if}
					<a
						href="/logs-list"
						class:font-bold={currentPath === '/logs-list'}
						class="block text-text-secondary hover:opacity-80"
						onclick={closeMobileMenu}
					>
						Logs
					</a>
					{#if isAdmin}
						<a
							href="/users-admin"
							class:font-bold={currentPath === '/users-admin'}
							class="block text-text-secondary hover:opacity-80"
							onclick={closeMobileMenu}
						>
							Users
						</a>
						<a
							href="/attendance-admin"
							class="block text-text-secondary hover:opacity-80"
							class:font-bold={currentPath === '/attendance-admin'}
							onclick={closeMobileMenu}
						>
							Attendance
						</a>
						<a
							href="/reports"
							class="block text-text-secondary hover:opacity-80"
							class:font-bold={currentPath === '/reports'}
							onclick={closeMobileMenu}
						>
							Reports
						</a>
						<a
							href="/templates-dashboard"
							class="block text-text-secondary hover:opacity-80"
							class:font-bold={currentPath === '/templates-dashboard'}
							onclick={closeMobileMenu}
						>
							Templates Dashboard
						</a>
					{/if}
					{#if isCompanyManager || isReadonlyHQ || data?.user?.role === 'logsmart_admin'}
						<a
							href="/branches"
							class="block text-text-secondary hover:opacity-80"
							class:font-bold={currentPath === '/branches'}
							onclick={closeMobileMenu}
						>
							Branches
						</a>
					{/if}
					<div class="border-t border-border-secondary pt-3">
						{#if data?.user}
							<div class="mb-2 text-sm text-text-secondary">
								{data.user.email}
							</div>
						{/if}
						<a
							href="/settings"
							class="mb-2 block rounded bg-bg-secondary px-4 py-2 text-text-primary hover:opacity-80"
							onclick={closeMobileMenu}
						>
							Settings
						</a>
						<button
							onclick={handleLogout}
							class="block w-full transform cursor-pointer rounded bg-bg-secondary px-4 py-2 text-left text-text-primary transition-all duration-150 hover:scale-105 hover:opacity-80"
						>
							Logout
						</button>
					</div>
				</nav>
			{/if}
		</div>
	</header>

	<main class="flex-1 overflow-auto">
		{@render children()}
	</main>
</div>
