<script lang="ts">
	import { page } from '$app/state';
	import type { LayoutData } from './$types';
	import '../app.css';
	import Icon from '$lib/assets/icon.svelte';
	import favicon from '$lib/assets/favicon.svg';

	let { children, data } = $props<{ children: any; data: LayoutData }>();

	const isAuthenticatedRoute = $derived(
		page.url.pathname.startsWith('/settings') || 
		page.url.pathname.startsWith('/dashboard') || 
		page.url.pathname.startsWith('/reports') || 
		page.url.pathname.startsWith('/templates-dashboard') ||
		page.url.pathname.startsWith('/log-template') ||
		page.url.pathname.startsWith('/users-admin')

	);

	async function handleLogout() {
		await fetch('/api/logout', { method: 'POST' });
		window.location.href = '/login';
	}
</script>

<svelte:head>
	<link rel="icon" href={favicon} />
</svelte:head>

<div class="flex h-screen flex-col">
	{#if !isAuthenticatedRoute}
		<header id="header" class="border-b border-gray-300 bg-white">
			<div class="mx-auto max-w-7xl px-6 py-2">
				<div class="flex items-center justify-between">
					<div class="flex items-center">
						<div class="h-12 w-12">
							<Icon />
						</div>
						<a href="/" class="text-2xl font-bold text-gray-900">LogSmart</a>
					</div>
					<nav class="hidden items-center gap-6 md:flex">
						<a href="/#features" class="text-gray-700 hover:text-gray-900">Features</a>
						<a href="/contact" class="text-gray-700 hover:text-gray-900">Contact</a>
					</nav>
					<div class="flex items-center gap-3">
						{#if data.isAuthenticated}
							<a
								href="/dashboard"
								class="rounded border border-gray-300 px-4 py-2 text-gray-700 hover:bg-gray-50"
							>
								Dashboard
							</a>
							<button
								onclick={handleLogout}
								class="rounded border border-gray-400 bg-gray-100 px-4 py-2 text-gray-900 hover:bg-gray-200"
							>
								Logout
							</button>
						{:else}
							<a
								href="/register-company"
								class="rounded border border-gray-300 px-4 py-2 text-gray-700 hover:bg-gray-50"
							>
								Register Company
							</a>
							<a
								href="/login"
								class="rounded border border-gray-400 bg-gray-100 px-4 py-2 text-gray-900 hover:bg-gray-200"
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
