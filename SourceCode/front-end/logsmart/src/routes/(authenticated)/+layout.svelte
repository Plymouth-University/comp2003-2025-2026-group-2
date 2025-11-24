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

<div class="min-h-screen bg-gray-50">
	<header class="border-b border-gray-200 bg-white shadow-sm">
		<div class="mx-auto max-w-7xl px-6 py-4">
			<div class="flex items-center justify-between">
				<div class="flex items-center gap-8">
					<a href="/dashboard" class="text-2xl font-bold text-blue-600">LogSmart</a>
					<nav class="flex items-center gap-6">
						<a 
							href="/dashboard" 
							class="text-gray-700 hover:text-gray-900"
							class:font-bold={currentPath === '/dashboard'}
							class:underline={currentPath === '/dashboard'}
						>
							Dashboard
						</a>
                        <a 
							href="/#logs" 
							class="text-gray-700 hover:text-gray-900"
						>
							Logs
						</a>
                        <a 
							href="/#users" 
							class="text-gray-700 hover:text-gray-900"
						>
							Users
						</a>
						<a 
							href="/reports" 
							class="text-gray-700 hover:text-gray-900"
							class:font-bold={currentPath === '/reports'}
							class:underline={currentPath === '/reports'}
						>
							Reports
						</a>
						<a 
							href="/templates-dashboard" 
							class="text-gray-700 hover:text-gray-900"
							class:font-bold={currentPath === '/templates-dashboard'}
							class:underline={currentPath === '/templates-dashboard'}
						>
							Templates
						</a>
					</nav>
				</div>
				<div class="flex items-center gap-4">
					{#if data?.user}
						<span class="text-sm text-gray-600">{data.user.email}</span>
					{/if}
					<a
						href="/settings"
						class="rounded bg-gray-100 px-4 py-2 text-gray-700 hover:bg-gray-200"
					>
						Settings
					</a>
					<button
						onclick={handleLogout}
						class="rounded bg-gray-100 px-4 py-2 text-gray-700 hover:bg-gray-200"
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
