<script lang="ts">
	import { api } from '$lib/api';
	import type { PageData } from './$types';

	const { data } = $props<{ data: PageData }>();
	let branches = $state([...data.branches]);
	let newBranchName = $state('');
	let newBranchAddress = $state('');
	let isSubmitting = $state(false);

	// Nominatim search states
	let searchQuery = $state('');
	let searchResults = $state<
		Array<{ display_name: string; lat: string; lon: string; type: string }>
	>([]);
	let isSearching = $state(false);
	let showResults = $state(false);
	let searchTimeout: ReturnType<typeof setTimeout> | null = null;

	async function searchLocations(query: string) {
		if (!query || query.length < 3) {
			searchResults = [];
			showResults = false;
			return;
		}

		isSearching = true;
		try {
			const response = await fetch(
				`https://nominatim.openstreetmap.org/search?q=${encodeURIComponent(query)}&format=json&limit=5&addressdetails=1`,
				{
					headers: {
						'Accept-Language': 'en',
						'User-Agent': 'LogSmart/1.0'
					}
				}
			);

			if (response.ok) {
				const data = await response.json();
				searchResults = data;
				showResults = data.length > 0;
			}
		} catch (error) {
			console.error('Error searching locations:', error);
		} finally {
			isSearching = false;
		}
	}

	function handleSearchInput(event: Event) {
		const target = event.target as HTMLInputElement;
		searchQuery = target.value;

		if (searchTimeout) {
			clearTimeout(searchTimeout);
		}

		searchTimeout = setTimeout(() => {
			searchLocations(searchQuery);
		}, 500);
	}

	function selectLocation(result: { display_name: string }) {
		newBranchAddress = result.display_name;
		searchQuery = result.display_name;
		showResults = false;
		searchResults = [];
	}

	function handleClickOutside(event: MouseEvent) {
		const target = event.target as HTMLElement;
		if (!target.closest('.search-container')) {
			showResults = false;
		}
	}

	async function handleAddBranch() {
		if (!newBranchName || !newBranchAddress) return;

		isSubmitting = true;
		try {
			const { data: branch, error } = await api.POST('/auth/company/branches', {
				body: {
					name: newBranchName,
					address: newBranchAddress
				}
			});

			if (error) {
				alert(`Failed to create branch: ${error.error}`);
			} else if (branch) {
				branches = [...branches, branch];
				newBranchName = '';
				newBranchAddress = '';
				searchQuery = '';
			}
		} catch (error) {
			console.error('Error creating branch:', error);
			alert('An unexpected error occurred');
		} finally {
			isSubmitting = false;
		}
	}

	// Edit branch states
	let editingBranchId = $state<string | null>(null);
	let editBranchName = $state('');
	let editBranchAddress = $state('');
	let editSearchQuery = $state('');
	let editSearchResults = $state<
		Array<{ display_name: string; lat: string; lon: string; type: string }>
	>([]);
	let isEditSearching = $state(false);
	let showEditResults = $state(false);
	let editSearchTimeout: ReturnType<typeof setTimeout> | null = null;
	let isUpdating = $state(false);

	function startEditingBranch(branch: { id: string; name: string; address: string }) {
		editingBranchId = branch.id;
		editBranchName = branch.name;
		editBranchAddress = branch.address;
		editSearchQuery = branch.address;
		editSearchResults = [];
		showEditResults = false;
	}

	function cancelEditingBranch() {
		editingBranchId = null;
		editBranchName = '';
		editBranchAddress = '';
		editSearchQuery = '';
		editSearchResults = [];
		showEditResults = false;
	}

	async function searchEditLocations(query: string) {
		if (!query || query.length < 3) {
			editSearchResults = [];
			showEditResults = false;
			return;
		}

		isEditSearching = true;
		try {
			const response = await fetch(
				`https://nominatim.openstreetmap.org/search?q=${encodeURIComponent(query)}&format=json&limit=5&addressdetails=1`,
				{
					headers: {
						'Accept-Language': 'en',
						'User-Agent': 'LogSmart/1.0'
					}
				}
			);

			if (response.ok) {
				const data = await response.json();
				editSearchResults = data;
				showEditResults = data.length > 0;
			}
		} catch (error) {
			console.error('Error searching locations:', error);
		} finally {
			isEditSearching = false;
		}
	}

	function handleEditSearchInput(event: Event) {
		const target = event.target as HTMLInputElement;
		editSearchQuery = target.value;

		if (editSearchTimeout) {
			clearTimeout(editSearchTimeout);
		}

		editSearchTimeout = setTimeout(() => {
			searchEditLocations(editSearchQuery);
		}, 500);
	}

	function selectEditLocation(result: { display_name: string }) {
		editBranchAddress = result.display_name;
		editSearchQuery = result.display_name;
		showEditResults = false;
		editSearchResults = [];
	}

	async function handleUpdateBranch() {
		if (!editingBranchId || !editBranchName || !editBranchAddress) return;

		isUpdating = true;
		try {
			const { data: updatedBranch, error } = await api.PUT('/auth/company/branches', {
				body: {
					branch_id: editingBranchId,
					name: editBranchName,
					address: editBranchAddress
				}
			});

			if (error) {
				alert(`Failed to update branch: ${error.error}`);
			} else if (updatedBranch) {
				branches = branches.map(b => b.id === editingBranchId ? updatedBranch : b);
				cancelEditingBranch();
			}
		} catch (error) {
			console.error('Error updating branch:', error);
			alert('An unexpected error occurred');
		} finally {
			isUpdating = false;
		}
	}

	// Delete branch states
	let showDeleteModal = $state(false);
	let branchToDelete = $state<{ id: string; name: string } | null>(null);
	let isRequestingDeletion = $state(false);
	let deletionMessage = $state<string | null>(null);

	function startDeleteBranch(branch: { id: string; name: string }) {
		branchToDelete = branch;
		showDeleteModal = true;
		deletionMessage = null;
	}

	function cancelDeleteBranch() {
		showDeleteModal = false;
		branchToDelete = null;
		deletionMessage = null;
	}

	async function requestBranchDeletion() {
		if (!branchToDelete) return;

		isRequestingDeletion = true;
		try {
			const response = await fetch('/api/auth/company/branches/request-deletion', {
				method: 'POST',
				headers: {
					'Content-Type': 'application/json'
				},
				body: JSON.stringify({ branch_id: branchToDelete.id })
			});

			const data = await response.json();

			if (!response.ok) {
				alert(`Failed to request deletion: ${data.error || 'Unknown error'}`);
			} else {
				deletionMessage = data.message || 'A confirmation email has been sent to your inbox.';
			}
		} catch (error) {
			console.error('Error requesting branch deletion:', error);
			alert('An unexpected error occurred');
		} finally {
			isRequestingDeletion = false;
		}
	}
</script>

<svelte:window onclick={handleClickOutside} />

<svelte:head>
	<title>Manage Branches - LogSmart</title>
</svelte:head>

<div class="mx-auto max-w-4xl p-6">
	<h1 class="mb-8 text-3xl font-bold text-text-primary">BRANCH MANAGEMENT</h1>

	<!-- Add Branch Form -->
	<div class="rounded-base mb-12 border-2 border-border-primary bg-bg-primary p-6 shadow-sm">
		<h2 class="mb-4 text-xl font-bold text-text-primary">ADD NEW BRANCH</h2>
		<form
			onsubmit={(e) => {
				e.preventDefault();
				handleAddBranch();
			}}
			class="flex flex-col gap-4 md:flex-row md:items-end"
		>
			<div class="flex-1">
				<label for="name" class="mb-2 block text-sm font-medium text-text-primary"
					>Branch Name</label
				>
				<input
					type="text"
					id="name"
					bind:value={newBranchName}
					class="rounded-base block w-full border-2 border-border-primary bg-bg-primary px-3 py-2 text-text-primary focus:ring-2 focus:outline-none"
					placeholder="e.g. London Office"
					required
				/>
			</div>
			<div class="search-container relative flex-1">
				<label for="address-search" class="mb-2 block text-sm font-medium text-text-primary">
					Address <span class="text-xs text-text-secondary"
						>(search for locations, POIs, or addresses)</span
					>
				</label>
				<input
					type="text"
					id="address-search"
					value={searchQuery}
					oninput={handleSearchInput}
					onfocus={() => {
						if (searchResults.length > 0) showResults = true;
					}}
					class="rounded-base block w-full border-2 border-border-primary bg-bg-primary px-3 py-2 text-text-primary focus:ring-2 focus:outline-none"
					placeholder="e.g. McDonald's London, 123 Regent St, or Tower Bridge"
					autocomplete="off"
				/>
				{#if isSearching}
					<div class="absolute top-[2.25rem] right-3 text-xs text-text-secondary">Searching...</div>
				{/if}
				{#if showResults && searchResults.length > 0}
					<div
						class="absolute z-50 mt-1 max-h-60 w-full overflow-auto rounded-md border-2 border-border-primary bg-bg-primary shadow-lg"
					>
						{#each searchResults as result}
							<button
								type="button"
								class="w-full px-4 py-2 text-left text-sm text-text-primary hover:bg-bg-secondary focus:bg-bg-secondary focus:outline-none"
								onclick={() => selectLocation(result)}
							>
								<div class="font-medium">{result.display_name.split(',')[0]}</div>
								<div class="truncate text-xs text-text-secondary">{result.display_name}</div>
							</button>
						{/each}
					</div>
				{/if}
				<input type="hidden" bind:value={newBranchAddress} required />
			</div>
			<button
				type="submit"
				disabled={isSubmitting || !newBranchName || !newBranchAddress}
				class="rounded-base border-2 border-border-primary bg-bg-secondary px-6 py-2 font-bold text-text-primary shadow-sm hover:translate-x-0.5 hover:translate-y-0.5 hover:shadow-none disabled:opacity-50"
			>
				{isSubmitting ? 'ADDING...' : 'ADD BRANCH'}
			</button>
		</form>
	</div>

	<!-- Branches List -->
	<div class="grid gap-6 md:grid-cols-2">
		{#each branches as branch}
			<div class="rounded-base border-2 border-border-primary bg-bg-primary p-6 shadow-sm">
				{#if editingBranchId === branch.id}
					<!-- Edit Mode -->
					<div class="space-y-4">
						<div>
							<label class="mb-1 block text-sm font-medium text-text-primary">Branch Name</label>
							<input
								type="text"
								bind:value={editBranchName}
								class="rounded-base block w-full border-2 border-border-primary bg-bg-primary px-3 py-2 text-text-primary focus:ring-2 focus:outline-none"
								placeholder="Branch name"
							/>
						</div>
						<div class="search-container relative">
							<label class="mb-1 block text-sm font-medium text-text-primary">Address</label>
							<input
								type="text"
								value={editSearchQuery}
								oninput={handleEditSearchInput}
								onfocus={() => {
									if (editSearchResults.length > 0) showEditResults = true;
								}}
								class="rounded-base block w-full border-2 border-border-primary bg-bg-primary px-3 py-2 text-text-primary focus:ring-2 focus:outline-none"
								placeholder="Search for address..."
								autocomplete="off"
							/>
							{#if isEditSearching}
								<div class="absolute top-[2.25rem] right-3 text-xs text-text-secondary">
									Searching...
								</div>
							{/if}
							{#if showEditResults && editSearchResults.length > 0}
								<div
									class="absolute z-50 mt-1 max-h-60 w-full overflow-auto rounded-md border-2 border-border-primary bg-bg-primary shadow-lg"
								>
									{#each editSearchResults as result}
										<button
											type="button"
											class="w-full px-4 py-2 text-left text-sm text-text-primary hover:bg-bg-secondary focus:bg-bg-secondary focus:outline-none"
											onclick={() => selectEditLocation(result)}
										>
											<div class="font-medium">{result.display_name.split(',')[0]}</div>
											<div class="truncate text-xs text-text-secondary">{result.display_name}</div>
										</button>
									{/each}
								</div>
							{/if}
						</div>
						<div class="flex gap-2 pt-2">
							<button
								type="button"
								onclick={handleUpdateBranch}
								disabled={isUpdating || !editBranchName || !editBranchAddress}
								class="rounded-base flex-1 border-2 border-border-primary bg-bg-secondary px-4 py-2 font-bold text-text-primary shadow-sm hover:translate-x-0.5 hover:translate-y-0.5 hover:shadow-none disabled:opacity-50"
							>
								{isUpdating ? 'SAVING...' : 'SAVE'}
							</button>
							<button
								type="button"
								onclick={cancelEditingBranch}
								disabled={isUpdating}
								class="rounded-base flex-1 border-2 border-border-primary bg-bg-primary px-4 py-2 font-bold text-text-primary shadow-sm hover:translate-x-0.5 hover:translate-y-0.5 hover:shadow-none disabled:opacity-50"
							>
								CANCEL
							</button>
						</div>
					</div>
				{:else}
					<!-- View Mode -->
					<div class="mb-2 flex items-center justify-between">
						<h3 class="text-xl font-bold text-text-primary">{branch.name}</h3>
						{#if branch.has_pending_deletion}
							<span class="rounded-full bg-yellow-100 px-2 py-1 text-xs font-semibold text-yellow-800">
								Deletion Pending
							</span>
						{:else}
							<span class="text-xs text-text-secondary">ID: {branch.id.slice(0, 8)}...</span>
						{/if}
					</div>
					<p class="text-text-secondary">{branch.address}</p>
					{#if branch.has_pending_deletion && branch.deletion_requested_at}
						<div class="mt-2 rounded bg-yellow-50 p-2 text-xs text-yellow-800">
							<span class="font-semibold">Deletion requested:</span> {new Date(branch.deletion_requested_at).toLocaleString()}
						</div>
					{/if}
					<div
						class="mt-4 flex items-center justify-between border-t-2 border-border-secondary pt-4"
					>
						<span class="text-xs font-medium text-text-secondary"
							>CREATED AT: {new Date(branch.created_at).toLocaleDateString()}</span
						>
						{#if !branch.has_pending_deletion}
							<div class="flex gap-2">
								<button
									type="button"
									onclick={() => startEditingBranch(branch)}
									class="rounded-base border-2 border-border-primary bg-bg-secondary px-4 py-1 text-sm font-bold text-text-primary shadow-sm hover:translate-x-0.5 hover:translate-y-0.5 hover:shadow-none"
								>
									EDIT
								</button>
								<button
									type="button"
									onclick={() => startDeleteBranch(branch)}
									class="rounded-base border-2 border-red-600 bg-red-200 px-4 py-1 text-sm font-bold shadow-sm hover:translate-x-0.5 hover:translate-y-0.5 hover:shadow-none"
								>
									DELETE
								</button>
							</div>
						{/if}
					</div>
				{/if}
			</div>
		{:else}
			<div
				class="rounded-base col-span-full border-2 border-dashed border-border-primary p-12 text-center"
			>
				<p class="text-text-secondary">No branches found. Add your first location above.</p>
			</div>
		{/each}
	</div>
</div>

{#if showDeleteModal && branchToDelete}
	<div
		class="fixed inset-0 z-50 flex items-center justify-center bg-black/50"
		onclick={(e) => e.target === e.currentTarget && cancelDeleteBranch()}
	>
		<div class="mx-4 w-full max-w-md rounded-lg border-2 p-6 shadow-xl" style="background-color: var(--bg-primary); border-color: var(--border-primary);">
			<div class="mb-4 text-center">
				<div class="mb-3 inline-flex h-12 w-12 items-center justify-center rounded-full" style="background-color: #fee2e2;">
					<svg class="h-6 w-6" style="color: #dc2626;" fill="none" stroke="currentColor" viewBox="0 0 24 24">
						<path stroke-linecap="round" stroke-linejoin="round" stroke-width="2" d="M12 9v2m0 4h.01m-6.938 4h13.856c1.54 0 2.502-1.667 1.732-3L13.732 4c-.77-1.333-2.694-1.333-3.464 0L3.34 16c-.77 1.333.192 3 1.732 3z" />
					</svg>
				</div>
				<h3 class="mb-2 text-xl font-bold" style="color: var(--text-primary);">Delete Branch</h3>
				<p class="text-sm" style="color: var(--text-secondary);">
					Are you sure you want to delete <strong>{branchToDelete.name}</strong>?
				</p>
			</div>

			{#if deletionMessage}
				<div class="mb-4 rounded-lg p-3 text-center text-sm" style="background-color: #dcfce7; color: #166534;">
					{deletionMessage}
				</div>
				<button
					type="button"
					onclick={cancelDeleteBranch}
					class="w-full rounded-lg py-2 font-semibold transition-opacity hover:opacity-80"
					style="background-color: var(--bg-secondary); color: var(--text-primary);"
				>
					Close
				</button>
			{:else}
				<div class="mb-4 rounded-lg border-l-4 p-3 text-sm" style="background-color: #fef3c7; border-color: #f59e0b; color: #92400e;">
					<p class="font-semibold mb-1">⚠️ This action requires email confirmation</p>
					<p>A confirmation link will be sent to your email address. You must click the link to complete the deletion.</p>
				</div>

				<div class="flex gap-3">
					<button
						type="button"
						onclick={cancelDeleteBranch}
						disabled={isRequestingDeletion}
						class="flex-1 rounded-lg border-2 py-2 font-semibold transition-opacity hover:opacity-80 disabled:opacity-50"
						style="border-color: var(--border-primary); color: var(--text-primary);"
					>
						Cancel
					</button>
					<button
						type="button"
						onclick={requestBranchDeletion}
						disabled={isRequestingDeletion}
						class="flex-1 rounded-lg py-2 font-semibold text-white transition-opacity hover:opacity-80 disabled:opacity-50"
						style="background-color: #dc2626;"
					>
						{#if isRequestingDeletion}
							<span class="inline-flex items-center">
								<svg class="mr-2 h-4 w-4 animate-spin" fill="none" viewBox="0 0 24 24">
									<circle class="opacity-25" cx="12" cy="12" r="10" stroke="currentColor" stroke-width="4"></circle>
									<path class="opacity-75" fill="currentColor" d="M4 12a8 8 0 018-8V0C5.373 0 0 5.373 0 12h4zm2 5.291A7.962 7.962 0 014 12H0c0 3.042 1.135 5.824 3 7.938l3-2.647z"></path>
								</svg>
								Sending...
							</span>
						{:else}
							Request Deletion
						{/if}
					</button>
				</div>
			{/if}
		</div>
	</div>
{/if}
