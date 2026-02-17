<script lang="ts">
	import { page } from '$app/state';
	import { goto } from '$app/navigation';
	import { api } from '$lib/api';

	let token = $derived(page.url.searchParams.get('token'));
	let isLoading = $state(true);
	let isDeleting = $state(false);
	let error = $state<string | null>(null);
	let success = $state<string | null>(null);
	let branchDetails = $state<{ name: string; id: string } | null>(null);

	$effect(() => {
		if (!token) {
			error = 'No confirmation token provided. Please check your email link.';
			isLoading = false;
			return;
		}

		// The token is validated when we try to confirm deletion
		// For now, we'll just show the confirmation UI
		isLoading = false;
	});

	async function confirmDeletion() {
		if (!token) return;

		isDeleting = true;
		error = null;
		success = null;

		try {
			const response = await fetch('/api/auth/company/branches/confirm-deletion', {
				method: 'POST',
				headers: {
					'Content-Type': 'application/json'
				},
				body: JSON.stringify({ token })
			});

			const data = await response.json();

			if (!response.ok) {
				error = data.error || 'Failed to delete branch. The token may have expired.';
			} else {
				success = data.message || 'Branch has been successfully deleted.';
			}
		} catch (err) {
			console.error('Error confirming deletion:', err);
			error = 'An unexpected error occurred. Please try again.';
		} finally {
			isDeleting = false;
		}
	}

	function goToBranches() {
		goto('/branches');
	}
</script>

<svelte:head>
	<title>Confirm Branch Deletion - LogSmart</title>
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
				Confirm Branch Deletion
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
				<p style="color: var(--text-secondary);">Loading...</p>
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
				onclick={goToBranches}
				class="w-full rounded-lg py-3 font-semibold transition-opacity hover:opacity-80"
				style="background-color: var(--bg-secondary); color: var(--text-primary);"
			>
				Return to Branches
			</button>
		{:else if error && !token}
			<div
				class="mb-6 rounded-lg p-4 text-center"
				style="background-color: #fee2e2; color: #dc2626;"
			>
				<p class="font-semibold">{error}</p>
			</div>
			<button
				onclick={goToBranches}
				class="w-full rounded-lg py-3 font-semibold transition-opacity hover:opacity-80"
				style="background-color: var(--bg-secondary); color: var(--text-primary);"
			>
				Return to Branches
			</button>
		{:else}
			<div
				class="mb-6 rounded-lg border-l-4 p-4"
				style="background-color: #fef3c7; border-color: #f59e0b;"
			>
				<p class="mb-2 font-semibold" style="color: #92400e;">⚠️ Important Notice</p>
				<p class="text-sm" style="color: #92400e;">Deleting this branch will:</p>
				<ul class="mt-2 ml-4 list-disc text-sm" style="color: #92400e;">
					<li>Permanently remove the branch from your company</li>
					<li>Disassociate any users currently assigned to this branch</li>
					<li>Delete all associated data and templates (if any)</li>
				</ul>
			</div>

			{#if error}
				<div
					class="mb-4 rounded-lg p-3 text-center text-sm"
					style="background-color: #fee2e2; color: #dc2626;"
				>
					{error}
				</div>
			{/if}

			<div class="flex gap-3">
				<button
					onclick={goToBranches}
					disabled={isDeleting}
					class="flex-1 rounded-lg border-2 py-3 font-semibold transition-opacity hover:opacity-80 disabled:opacity-50"
					style="border-color: var(--border-primary); color: var(--text-primary);"
				>
					Cancel
				</button>
				<button
					onclick={confirmDeletion}
					disabled={isDeleting}
					class="flex-1 rounded-lg py-3 font-semibold text-white transition-opacity hover:opacity-80 disabled:opacity-50"
					style="background-color: #dc2626;"
				>
					{#if isDeleting}
						<span class="inline-flex items-center">
							<svg class="mr-2 h-4 w-4 animate-spin" fill="none" viewBox="0 0 24 24">
								<circle
									class="opacity-25"
									cx="12"
									cy="12"
									r="10"
									stroke="currentColor"
									stroke-width="4"
								></circle>
								<path
									class="opacity-75"
									fill="currentColor"
									d="M4 12a8 8 0 018-8V0C5.373 0 0 5.373 0 12h4zm2 5.291A7.962 7.962 0 014 12H0c0 3.042 1.135 5.824 3 7.938l3-2.647z"
								></path>
							</svg>
							Deleting...
						</span>
					{:else}
						Delete Branch
					{/if}
				</button>
			</div>
		{/if}
	</div>
</div>
