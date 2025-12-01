<script lang="ts">
	import { onMount } from 'svelte';
	import { page } from '$app/state';
	import type { LayoutData } from './$types';
	import '../app.css';
	import Icon from '$lib/assets/icon.svelte';
	import favicon from '$lib/assets/favicon.svg';
	import { isDarkMode } from '$lib/stores/theme';

	let { children, data } = $props<{ children: any; data: LayoutData }>();

	const isAuthenticatedRoute = $derived(
		page.url.pathname.startsWith('/dashboard') ||
			page.url.pathname.startsWith('/log-template') ||
			page.url.pathname.startsWith('/logs-list') ||
			page.url.pathname.startsWith('/reports') ||
			page.url.pathname.startsWith('/settings') ||
			page.url.pathname.startsWith('/templates-dashboard') ||
			page.url.pathname.startsWith('/users-admin')
	);

	async function handleLogout() {
		await fetch('/api/logout', { method: 'POST' });
		window.location.href = '/login';
	}

	onMount(() => {
		isDarkMode.initialize();
	});
</script>

<svelte:head>
	<link rel="icon" href={favicon} />
</svelte:head>

<div class="flex h-screen flex-col">
	{#if !isAuthenticatedRoute}
		<header id="header" style="border-color: var(--border-secondary); background-color: var(--bg-primary);" class="border-b">
			<div class="mx-auto max-w-7xl px-6 py-2">
				<div class="flex items-center justify-between">
					<div class="flex items-center">
						<div class="h-12 w-12">
							<Icon />
						</div>
						<a href="/" style="color: var(--text-primary);" class="text-2xl font-bold">LogSmart</a>
					</div>
					<nav class="hidden items-center gap-6 md:flex">
						<a href="/#features" style="color: var(--text-secondary);" class="hover:opacity-80">Features</a>
						<a href="/contact" style="color: var(--text-secondary);" class="hover:opacity-80">Contact</a>
					</nav>
					<div class="flex items-center gap-3">
						{#if data.isAuthenticated}
							<a
								href="/dashboard"
								style="border-color: var(--border-secondary); color: var(--text-secondary); background-color: var(--bg-secondary);"
								class="rounded border px-4 py-2 hover:opacity-80"
							>
								Dashboard
							</a>
							<button
								onclick={handleLogout}
								style="border-color: var(--border-primary); background-color: var(--bg-secondary); color: var(--text-primary);"
								class="rounded border px-4 py-2 hover:opacity-80"
							>
								Logout
							</button>
						{:else}
							<a
								href="/register-company"
								style="border-color: var(--border-secondary); color: var(--text-secondary); background-color: var(--bg-secondary);"
								class="rounded border px-4 py-2 hover:opacity-80"
							>
								Register Company
							</a>
							<a
								href="/login"
								style="border-color: var(--border-primary); background-color: var(--bg-secondary); color: var(--text-primary);"
								class="rounded border px-4 py-2 hover:opacity-80"
							>
								Login
							</a>
						{/if}
					</div>
				</div>
			</div>
		</header>
	{/if}

	<main class="flex w-full flex-1">
		{@render children()}
	</main>
</div>
