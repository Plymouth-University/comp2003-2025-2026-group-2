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
	<meta name="robots" content="index,follow" />
	<meta name="theme-color" content="#0b172a" />
	<meta property="og:site_name" content="LogSmart" />
	<meta property="og:type" content="website" />
	<meta property="og:title" content="LogSmart - Digital Logging System" />
	<meta
		property="og:description"
		content="Replace paper logs with LogSmart. Secure, cloud-based digital logging for the food service industry."
	/>
	<meta property="og:url" content="https://logsmart.app" />
	<meta property="og:image" content="https://logsmart.app/og-image.jpg" />
	<meta property="og:image:width" content="1200" />
	<meta property="og:image:height" content="630" />
	<meta name="twitter:card" content="summary_large_image" />
	<meta name="twitter:title" content="LogSmart - Digital Logging System" />
	<meta
		name="twitter:description"
		content="Replace paper logs with LogSmart. Secure, cloud-based digital logging for the food service industry."
	/>
	<meta name="twitter:image" content="https://logsmart.app/og-image.jpg" />
	<script type="application/ld+json">
		{
			"@context": "https://schema.org",
			"@type": "Organization",
			"name": "LogSmart",
			"url": "https://logsmart.app",
			"logo": "https://logsmart.app/favicon.svg"
		}
	</script>
	<script type="application/ld+json">
		{
			"@context": "https://schema.org",
			"@type": "FAQPage",
			"mainEntity": [
				{
					"@type": "Question",
					"name": "How does LogSmart help with audits?",
					"acceptedAnswer": {
						"@type": "Answer",
						"text": "LogSmart keeps tamper-proof digital records so audits are faster and easier."
					}
				},
				{
					"@type": "Question",
					"name": "Can I customize templates?",
					"acceptedAnswer": {
						"@type": "Answer",
						"text": "Yes. Use built-in templates or customize them to match your operation."
					}
				},
				{
					"@type": "Question",
					"name": "Is my data secure?",
					"acceptedAnswer": {
						"@type": "Answer",
						"text": "Data is stored securely in the cloud with access controls for your team."
					}
				}
			]
		}
	</script>
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
								class="rounded border border-border-secondary bg-bg-secondary px-4 py-2 ml-3 whitespace-nowrap text-text-secondary hover:opacity-80"
							>
								<span class="hidden sm:inline">Register Company</span>
								<span class="sm:hidden">Register</span>
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
