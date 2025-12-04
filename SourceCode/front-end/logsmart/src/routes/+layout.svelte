<script lang="ts">
	import { onMount } from 'svelte';
	import { page } from '$app/state';
	import type { LayoutData } from './$types';
	import '../app.css';
	import Icon from '$lib/assets/icon.svelte';
	import { isDarkMode } from '$lib/stores/theme';

	let { children, data } = $props<{ children: any; data: LayoutData }>();

	const isAuthenticatedRoute = $derived(
		page.url.pathname.startsWith('/dashboard') ||
			page.url.pathname.startsWith('/log-template') ||
			page.url.pathname.startsWith('/logs-list') ||
			page.url.pathname.startsWith('/reports') ||
			page.url.pathname.startsWith('/settings') ||
			page.url.pathname.startsWith('/templates-dashboard') ||
			page.url.pathname.startsWith('/users-admin') ||
			page.url.pathname.startsWith('/template-designer')
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
	<meta
		name="description"
		content="LogSmart - Digital logging system for the food service industry. Secure, eco-friendly, and audit-ready."
	/>
	<meta property="og:site_name" content="LogSmart" />
	<meta property="og:type" content="website" />
	<meta property="og:title" content="LogSmart - Digital Logging System" />
	<meta
		property="og:description"
		content="Replace paper logs with LogSmart. Secure, cloud-based digital logging for the food service industry."
	/>
	<meta property="og:url" content="https://logsmart.app" />
	<meta name="twitter:card" content="summary_large_image" />
	<meta name="twitter:title" content="LogSmart - Digital Logging System" />
	<meta
		name="twitter:description"
		content="Replace paper logs with LogSmart. Secure, cloud-based digital logging for the food service industry."
	/>
</svelte:head>

<div class="flex h-screen flex-col">
	{#if !isAuthenticatedRoute}
		<header id="header" class="border-b border-border-secondary bg-bg-primary">
			<div class="mx-auto max-w-7xl px-6 py-2">
				<div class="flex items-center justify-between">
					<div class="flex items-center">
						<div class="h-12 w-12">
							<Icon />
						</div>
						<a href="/" class="text-2xl font-bold text-text-primary">LogSmart</a>
					</div>
					<nav class="hidden items-center gap-6 md:flex">
						<a href="/#features" class="text-text-secondary hover:opacity-80">Features</a>
						<a href="/contact" class="text-text-secondary hover:opacity-80">Contact</a>
					</nav>
					<div class="flex items-center gap-3">
						{#if data.isAuthenticated}
							<a
								href="/dashboard"
								class="rounded border border-border-secondary bg-bg-secondary px-4 py-2 text-text-secondary hover:opacity-80"
							>
								Dashboard
							</a>
							<button
								onclick={handleLogout}
								class="rounded border border-border-primary bg-bg-secondary px-4 py-2 text-text-primary hover:opacity-80"
							>
								Logout
							</button>
						{:else}
							<a
								href="/register-company"
								class="rounded border border-border-secondary bg-bg-secondary px-4 py-2 text-text-secondary hover:opacity-80"
							>
								Register Company
							</a>
							<a
								href="/login"
								class="rounded border border-border-primary bg-bg-secondary px-4 py-2 text-text-primary hover:opacity-80"
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
