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
				<p style="color: var(--text-secondary);">Validating confirmation link...</p>
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
		{:else if companyName}
			<div class="space-y-4">
				<p class="text-center" style="color: var(--text-secondary);">
					To confirm deletion, please type the company name exactly as shown:
				</p>
				<div
					class="rounded-lg p-4 text-center"
					style="background-color: var(--bg-secondary); border: 1px solid var(--border-primary);"
				>
					<p class="font-mono text-lg font-bold" style="color: var(--text-primary);">
						{companyName}
					</p>
				</div>
				<div>
					<label
						for="companyNameInput"
						class="mb-1 block text-sm font-medium"
						style="color: var(--text-secondary);"
					>
						Type company name to confirm
					</label>
					<input
						id="companyNameInput"
						type="text"
						bind:value={typedCompanyName}
						placeholder="Enter company name"
						class="w-full rounded-lg border p-3"
						style="background-color: var(--bg-primary); color: var(--text-primary); border-color: var(--border-primary);"
					/>
				</div>
				<button
					onclick={confirmDeletion}
					disabled={!isNameValid || deletionStarted}
					class="w-full rounded-lg py-3 font-semibold transition-opacity disabled:opacity-50"
					style="background-color: #dc2626; color: white;"
				>
					{deletionStarted ? 'Deleting...' : 'Delete Company'}
				</button>
				<button
					onclick={goToHome}
					class="w-full rounded-lg py-3 font-semibold transition-opacity hover:opacity-80"
					style="background-color: var(--bg-secondary); color: var(--text-primary);"
				>
					Cancel
				</button>
			</div>
		{/if}
	</div>
</div>
