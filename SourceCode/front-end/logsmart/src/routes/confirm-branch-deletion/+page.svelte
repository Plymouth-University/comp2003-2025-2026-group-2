<script lang="ts">
	import { page } from '$app/state';
	import { goto } from '$app/navigation';

	let token = $derived(page.url.searchParams.get('token'));
	let isLoading = $state(true);
	let isDeleting = $state(false);
	let error = $state<string | null>(null);
	let success = $state<string | null>(null);

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

<div class="flex min-h-full w-full items-center justify-center bg-bg-secondary p-6">
	<div
		class="w-full max-w-md rounded-lg border-2 border-border-primary bg-bg-primary p-8 shadow-lg"
	>
		<div class="mb-6 text-center">
			<div class="mb-4 inline-flex h-16 w-16 items-center justify-center rounded-full bg-error-bg">
				<svg
					class="h-8 w-8 text-button-secondary"
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
			<h1 class="mb-2 text-2xl font-bold text-text-primary">Confirm Branch Deletion</h1>
			<p class="text-sm text-text-secondary">This is a permanent action that cannot be undone.</p>
		</div>

		{#if isLoading}
			<div class="py-8 text-center">
				<div
					class="mx-auto mb-4 h-8 w-8 animate-spin rounded-full border-b-2 border-text-primary"
				></div>
				<p class="text-text-secondary">Loading...</p>
			</div>
		{:else if success}
			<div class="mb-6 rounded-lg bg-clock-in-bg p-4 text-center text-create-button">
				<p class="mb-2 font-semibold">Success!</p>
				<p class="text-sm">{success}</p>
			</div>
			<button
				onclick={goToBranches}
				class="w-full rounded-lg bg-bg-secondary py-3 font-semibold text-text-primary transition-opacity hover:opacity-80"
			>
				Return to Branches
			</button>
		{:else if error && !token}
			<div class="mb-6 rounded-lg bg-error-bg p-4 text-center text-button-secondary">
				<p class="font-semibold">{error}</p>
			</div>
			<button
				onclick={goToBranches}
				class="w-full rounded-lg bg-bg-secondary py-3 font-semibold text-text-primary transition-opacity hover:opacity-80"
			>
				Return to Branches
			</button>
		{:else}
			<div class="mb-6 rounded-lg border-l-4 border-orange bg-orange-light p-4">
				<p class="mb-2 font-semibold text-orange-dark">⚠️ Important Notice</p>
				<p class="text-sm text-orange-dark">Deleting this branch will:</p>
				<ul class="mt-2 ml-4 list-disc text-sm text-orange-dark">
					<li>Permanently remove the branch from your company</li>
					<li>Disassociate any users currently assigned to this branch</li>
					<li>Delete all associated data and templates (if any)</li>
				</ul>
			</div>

			{#if error}
				<div class="mb-4 rounded-lg bg-error-bg p-3 text-center text-sm text-button-secondary">
					{error}
				</div>
			{/if}

			<div class="flex gap-3">
				<button
					onclick={goToBranches}
					disabled={isDeleting}
					class="flex-1 rounded-lg border-2 border-border-primary py-3 font-semibold text-text-primary transition-opacity hover:opacity-80 disabled:opacity-50"
				>
					Cancel
				</button>
				<button
					onclick={confirmDeletion}
					disabled={isDeleting}
					class="flex-1 rounded-lg bg-delete-button-active py-3 font-semibold text-white transition-opacity hover:opacity-80 disabled:opacity-50"
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
