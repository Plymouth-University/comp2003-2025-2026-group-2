<script lang="ts">
	import { page } from '$app/state';
	import { goto } from '$app/navigation';

	let companyId = $derived(page.url.searchParams.get('company_id'));
	let token = $derived(page.url.searchParams.get('token'));
	let isLoading = $state(true);
	let error = $state<string | null>(null);
	let success = $state<string | null>(null);
	let companyName = $state<string | null>(null);
	let typedCompanyName = $state('');
	let deletionStarted = $state(false);

	$effect(() => {
		if (!companyId || !token) {
			error = 'Missing company ID or confirmation token. Please check your email link.';
			isLoading = false;
			return;
		}

		validateTokenAndGetCompanyName();
	});

	async function validateTokenAndGetCompanyName() {
		if (!companyId || !token) return;
		error = null;

		try {
			const response = await fetch(
				`/api/companies/${companyId}/validate-deletion-token?token=${encodeURIComponent(token)}`
			);

			const data = await response.json();

			if (!response.ok) {
				error = data.error || 'Failed to validate token. The token may have expired.';
			} else {
				companyName = data.companyName;
			}
		} catch (err) {
			console.error('Error validating token:', err);
			error = 'An unexpected error occurred. Please try again.';
		} finally {
			isLoading = false;
		}
	}

	async function confirmDeletion() {
		if (!companyId || !token) return;
		deletionStarted = true;
		error = null;
		success = null;

		try {
			const response = await fetch(`/api/companies/${companyId}/confirm-deletion`, {
				method: 'POST',
				headers: { 'Content-Type': 'application/json' },
				body: JSON.stringify({ token })
			});

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
			deletionStarted = false;
		}
	}

	function goToHome() {
		goto('/');
	}

	let isNameValid = $derived(
		companyName !== null && typedCompanyName.trim() === companyName.trim()
	);
</script>

<svelte:head>
	<title>Confirm Company Deletion - LogSmart</title>
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
			<h1 class="mb-2 text-2xl font-bold text-text-primary">Confirm Company Deletion</h1>
			<p class="text-sm text-text-secondary">This is a permanent action that cannot be undone.</p>
		</div>

		{#if isLoading}
			<div class="py-8 text-center">
				<div
					class="mx-auto mb-4 h-8 w-8 animate-spin rounded-full border-b-2 border-text-primary"
				></div>
				<p class="text-text-secondary">Validating confirmation link...</p>
			</div>
		{:else if success}
			<div class="mb-6 rounded-lg bg-clock-in-bg p-4 text-center text-create-button">
				<p class="mb-2 font-semibold">Success!</p>
				<p class="text-sm">{success}</p>
			</div>
			<button
				onclick={goToHome}
				class="w-full rounded-lg bg-bg-secondary py-3 font-semibold text-text-primary transition-opacity hover:opacity-80"
			>
				Return to Home
			</button>
		{:else if error}
			<div class="mb-6 rounded-lg bg-error-bg p-4 text-center text-button-secondary">
				<p class="font-semibold">Error</p>
				<p class="text-sm">{error}</p>
			</div>
			<button
				onclick={goToHome}
				class="w-full rounded-lg bg-bg-secondary py-3 font-semibold text-text-primary transition-opacity hover:opacity-80"
			>
				Return to Home
			</button>
		{:else if companyName}
			<div class="space-y-4">
				<p class="text-center text-text-secondary">
					To confirm deletion, please type the company name exactly as shown:
				</p>
				<div class="rounded-lg border border-border-primary bg-bg-secondary p-4 text-center">
					<p class="font-mono text-lg font-bold text-text-primary">
						{companyName}
					</p>
				</div>
				<div>
					<label for="companyNameInput" class="mb-1 block text-sm font-medium text-text-secondary">
						Type company name to confirm
					</label>
					<input
						id="companyNameInput"
						type="text"
						bind:value={typedCompanyName}
						placeholder="Enter company name"
						class="w-full rounded-lg border border-border-primary bg-bg-primary p-3 text-text-primary"
					/>
				</div>
				<button
					onclick={confirmDeletion}
					disabled={!isNameValid || deletionStarted}
					class="w-full rounded-lg bg-button-secondary py-3 font-semibold text-bg-primary transition-opacity disabled:opacity-50"
				>
					{deletionStarted ? 'Deleting...' : 'Delete Company'}
				</button>
				<button
					onclick={goToHome}
					class="w-full rounded-lg bg-bg-secondary py-3 font-semibold text-text-primary transition-opacity hover:opacity-80"
				>
					Cancel
				</button>
			</div>
		{/if}
	</div>
</div>
