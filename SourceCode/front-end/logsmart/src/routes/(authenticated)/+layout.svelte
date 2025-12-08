<script lang="ts">
	import { page } from '$app/state';
	import type { LayoutData } from './$types';

	let { data, children } = $props<{ data: LayoutData; children: any }>();

	let mobileMenuOpen = $state(false);

	const currentPath = $derived(page.url.pathname);
	const isAdmin = $derived(data?.user?.role === 'admin');

	async function handleLogout() {
		await fetch('/api/logout', { method: 'POST' });
		window.location.href = '/login';
	}

	function closeMobileMenu() {
		mobileMenuOpen = false;
	}
</script>

<div class="flex h-screen w-full flex-col" style="background-color: var(--bg-secondary);">
	<header
		style="border-color: var(--border-secondary); background-color: var(--bg-primary);"
		class="shrink-0 border-b shadow-sm"
	>
		<div class="mx-auto max-w-7xl px-6 py-4">
			<div class="flex items-center justify-between">
				<div class="flex items-center gap-8">
					<a
						href={isAdmin ? '/dashboard' : '/logs-list'}
						class="text-2xl font-bold"
						style="color: #94C5CC;">LogSmart</a
					>
					<nav class="hidden flex-wrap items-center gap-4 md:flex md:gap-6">
						{#if isAdmin}
							<a
								href="/dashboard"
								class="hover:opacity-80"
								style="color: var(--text-secondary);"
								class:font-bold={currentPath === '/dashboard'}
								class:underline={currentPath === '/dashboard'}
							>
								Dashboard
							</a>
						{/if}
						<a
							href="/logs-list"
							style="color: var(--text-secondary);"
							class:font-bold={currentPath === '/logs-list'}
							class:underline={currentPath === '/logs-list'}
							class="hover:opacity-80"
						>
							Logs
						</a>
						{#if isAdmin}
							<a
								href="/users-admin"
								style="color: var(--text-secondary);"
								class:font-bold={currentPath === '/users-admin'}
								class:underline={currentPath === '/users-admin'}
								class="hover:opacity-80"
							>
								Users
							</a>
							<a
								href="/reports"
								style="color: var(--text-secondary);"
								class="hover:opacity-80"
								class:font-bold={currentPath === '/reports'}
								class:underline={currentPath === '/reports'}
							>
								Reports
							</a>
							<a
								href="/templates-dashboard"
								class="hover:opacity-80"
								style="color: var(--text-secondary);"
								class:font-bold={currentPath === '/templates-dashboard'}
								class:underline={currentPath === '/templates-dashboard'}
							>
								Templates Dashboard
							</a>
						{/if}
					</nav>
				</div>
				<div class="flex items-center gap-4">
					<div class="hidden flex-wrap items-center gap-4 md:flex">
						{#if data?.user}
							<span class="text-sm" style="color: var(--text-secondary);">{data.user.email}</span>
						{/if}
						<a
							href="/settings"
							class="rounded px-4 py-2 hover:opacity-80"
							style="background-color: var(--bg-secondary); color: var(--text-primary);"
						>
							Settings
						</a>
						<button
							onclick={handleLogout}
							class="rounded px-4 py-2 hover:opacity-80"
							style="background-color: var(--bg-secondary); color: var(--text-primary);"
						>
							Logout
						</button>
					</div>
					<button
						onclick={() => (mobileMenuOpen = !mobileMenuOpen)}
						class="p-2 hover:opacity-80 md:hidden"
						style="color: var(--text-secondary);"
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
				<nav
					class="mt-4 flex flex-col gap-3 border-t pt-4 md:hidden"
					style="border-color: var(--border-secondary);"
				>
					{#if isAdmin}
						<a
							href="/dashboard"
							class="block hover:opacity-80"
							style="color: var(--text-secondary);"
							class:font-bold={currentPath === '/dashboard'}
							onclick={closeMobileMenu}
						>
							Dashboard
						</a>
					{/if}
					<a
						href="/logs-list"
						style="color: var(--text-secondary);"
						class:font-bold={currentPath === '/logs-list'}
						class="block hover:opacity-80"
						onclick={closeMobileMenu}
					>
						Logs
					</a>
					{#if isAdmin}
						<a
							href="/users-admin"
							style="color: var(--text-secondary);"
							class:font-bold={currentPath === '/users-admin'}
							class="block hover:opacity-80"
							onclick={closeMobileMenu}
						>
							Users
						</a>
						<a
							href="/reports"
							style="color: var(--text-secondary);"
							class="block hover:opacity-80"
							class:font-bold={currentPath === '/reports'}
							onclick={closeMobileMenu}
						>
							Reports
						</a>
						<a
							href="/templates-dashboard"
							class="block hover:opacity-80"
							style="color: var(--text-secondary);"
							class:font-bold={currentPath === '/templates-dashboard'}
							onclick={closeMobileMenu}
						>
							Templates Dashboard
						</a>
					{/if}
					<div class="border-t pt-3" style="border-color: var(--border-secondary);">
						{#if data?.user}
							<div class="mb-2 text-sm" style="color: var(--text-secondary);">
								{data.user.email}
							</div>
						{/if}
						<a
							href="/settings"
							class="mb-2 block rounded px-4 py-2 hover:opacity-80"
							style="background-color: var(--bg-secondary); color: var(--text-primary);"
							onclick={closeMobileMenu}
						>
							Settings
						</a>
						<button
							onclick={handleLogout}
							class="block w-full rounded px-4 py-2 text-left hover:opacity-80"
							style="background-color: var(--bg-secondary); color: var(--text-primary);"
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
