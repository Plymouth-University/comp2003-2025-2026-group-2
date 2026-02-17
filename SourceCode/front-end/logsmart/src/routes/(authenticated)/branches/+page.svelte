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
				branches = branches.map((b) => (b.id === editingBranchId ? updatedBranch : b));
				cancelEditingBranch();
			}
		} catch (error) {
			console.error('Error updating branch:', error);
			alert('An unexpected error occurred');
		} finally {
			isUpdating = false;
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
						<span class="text-xs text-text-secondary">ID: {branch.id.slice(0, 8)}...</span>
					</div>
					<p class="text-text-secondary">{branch.address}</p>
					<div
						class="mt-4 flex items-center justify-between border-t-2 border-border-secondary pt-4"
					>
						<span class="text-xs font-medium text-text-secondary"
							>CREATED AT: {new Date(branch.created_at).toLocaleDateString()}</span
						>
						<button
							type="button"
							onclick={() => startEditingBranch(branch)}
							class="rounded-base border-2 border-border-primary bg-bg-secondary px-4 py-1 text-sm font-bold text-text-primary shadow-sm hover:translate-x-0.5 hover:translate-y-0.5 hover:shadow-none"
						>
							EDIT
						</button>
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
