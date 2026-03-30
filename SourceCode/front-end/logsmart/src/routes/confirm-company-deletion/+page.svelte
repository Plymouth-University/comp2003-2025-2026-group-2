<script lang="ts">
	import { page } from '$app/state';
	import { goto } from '$app/navigation';

	let companyId = $derived(page.url.searchParams.get('company_id'));
	let token = $derived(page.url.searchParams.get('token'));
	let isLoading = $state(true);
	let isDeleting = $state(false);
	let error = $state<string | null>(null);
	let success = $state<string | null>(null);

	$effect(() => {
		if (!companyId || !token) {
			error = 'Missing company ID or confirmation token. Please check your email link.';
			isLoading = false;
			return;
		}

		confirmDeletion();
	});

	async function confirmDeletion() {
		if (!companyId || !token) return;

		isDeleting = true;
		error = null;
		success = null;

		try {
			const response = await fetch(
				`/api/companies/${companyId}/confirm-deletion?token=${encodeURIComponent(token)}`,
				{
					method: 'GET'
				}
			);

			const data = await response.json();

			if (!response.ok) {
				error = data.error || 'Failed to delete company. The token may have expired.';
			} else {
				success =
					data.message ||
					'Company has been successfully deleted. Data will be retained for 30 days.';
			}
		} catch (err) {
			console.error('Error confirming deletion:', err);
			error = 'An unexpected error occurred. Please try again.';
		} finally {
			isDeleting = false;
			isLoading = false;
		}
	}

	function goToHome() {
		goto('/');
	}
</script>

<svelte:head>
	<title>Confirm Company Deletion - LogSmart</title>
</svelte:head>

<div
	class="flex min-h-full w-full items-center justify-center p-6"
	style="background-color: var(--bg-secondary);"
>
	<div
		class="w-full max-w-md rounded-lg border-2 p-8 shadow-lg"
		style="background-color: var(--bg-primary); border-color: var(--border-primary);"
	>
		<div class="mb-6 text-center">
			<div
				class="mb-4 inline-flex h-16 w-16 items-center justify-center rounded-full"
				style="background-color: #fee2e2;"
			>
				<svg
					class="h-8 w-8"
					style="color: #dc2626;"
					fill="none"
					stroke="currentColor"
					viewBox="0 0 24 24"
				>
					<path
						stroke-linecap="round"
						stroke-linejoin="round"
						stroke-width="2"
						d="M12 9v2m0 4h.01m-6.938 4h13.856c1.54 0 2.502-1.667 1.732-3L13.732 4c-.77-1.333-2.694-1.333-3.464 0L3.34 16c-.77 1.333.192 3 1.732 3z"
					/>
				</svg>
			</div>
			<h1 class="mb-2 text-2xl font-bold" style="color: var(--text-primary);">
				Confirm Company Deletion
			</h1>
			<p class="text-sm" style="color: var(--text-secondary);">
				This is a permanent action that cannot be undone.
			</p>
		</div>

		{#if isLoading}
			<div class="py-8 text-center">
				<div
					class="mx-auto mb-4 h-8 w-8 animate-spin rounded-full border-b-2"
					style="border-color: var(--text-primary);"
				></div>
				<p style="color: var(--text-secondary);">Confirming deletion...</p>
			</div>
		{:else if success}
			<div
				class="mb-6 rounded-lg p-4 text-center"
				style="background-color: #dcfce7; color: #166534;"
			>
				<p class="mb-2 font-semibold">Success!</p>
				<p class="text-sm">{success}</p>
			</div>
			<button
				onclick={goToHome}
				class="w-full rounded-lg py-3 font-semibold transition-opacity hover:opacity-80"
				style="background-color: var(--bg-secondary); color: var(--text-primary);"
			>
				Return to Home
			</button>
		{:else if error}
			<div
				class="mb-6 rounded-lg p-4 text-center"
				style="background-color: #fee2e2; color: #dc2626;"
			>
				<p class="font-semibold">Error</p>
				<p class="text-sm">{error}</p>
			</div>
			<button
				onclick={goToHome}
				class="w-full rounded-lg py-3 font-semibold transition-opacity hover:opacity-80"
				style="background-color: var(--bg-secondary); color: var(--text-primary);"
			>
				Return to Home
			</button>
		{/if}
	</div>
</div>
