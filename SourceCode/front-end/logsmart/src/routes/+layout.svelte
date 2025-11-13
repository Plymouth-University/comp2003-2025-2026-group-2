<script lang="ts">
	import '../app.css';
	import favicon from '$lib/assets/favicon.svg';

	let { children } = $props();
</script>

<svelte:head>
	<link rel="icon" href={favicon} />
</svelte:head>

<header class="border-b border-gray-300 bg-white">
	<div class="mx-auto max-w-7xl px-6 py-4">
		<div class="flex items-center justify-between">
			<a href="/" class="text-2xl font-bold text-gray-900">LogSmart</a>
			<nav class="hidden items-center gap-6 md:flex">
				<a href="/#features" class="text-gray-700 hover:text-gray-900">Features</a>
				<a href="/contact" class="text-gray-700 hover:text-gray-900">Contact</a>
			</nav>
			<div class="flex items-center gap-3">
				<a href="/register-company" class="rounded border border-gray-300 px-4 py-2 text-gray-700 hover:bg-gray-50">
					Register Company
				</a>
				<a href="/login" class="rounded border border-gray-400 bg-gray-100 px-4 py-2 text-gray-900 hover:bg-gray-200">
					Login
				</a>
			</div>
		</div>
	</div>
</header>

{@render children()}
