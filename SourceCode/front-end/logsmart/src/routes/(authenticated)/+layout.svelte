<script lang="ts">
	import { page } from '$app/state';
	import type { LayoutData } from './$types';

	let { data, children } = $props<{ data: LayoutData; children: any }>();

	const currentPath = $derived(page.url.pathname);

	async function handleLogout() {
		await fetch('/api/logout', { method: 'POST' });
		window.location.href = '/login';
	}
</script>

<div class="min-h-screen w-full" style="background-color: var(--bg-secondary);">
	<header
		style="border-color: var(--border-secondary); background-color: var(--bg-primary);"
		class="border-b shadow-sm"
	>
		<div class="mx-auto max-w-7xl px-6 py-4">
			<div class="flex items-center justify-between">
				<div class="flex items-center gap-8">
					<a href="/dashboard" class="text-2xl font-bold" style="color: #94C5CC;">LogSmart</a>
					<nav class="flex items-center gap-6">
						<a
							href="/dashboard"
							class="hover:opacity-80"
							style="color: var(--text-secondary);"
							class:font-bold={currentPath === '/dashboard'}
							class:underline={currentPath === '/dashboard'}
						>
							Dashboard
						</a>
						<a href="/logs-list" style="color: var(--text-secondary);" class="hover:opacity-80">
							Logs
						</a>
						<a href="/user-admin" style="color: var(--text-secondary);" class="hover:opacity-80">
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
					</nav>
				</div>
				<div class="flex items-center gap-4">
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
			</div>
		</div>
	</header>

	<main>
		{@render children()}
	</main>
</div>
