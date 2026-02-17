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
	let searchResults = $state<Array<{ display_name: string; lat: string; lon: string; type: string }>>([]);
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
			<div class="flex-1 search-container relative">
				<label for="address-search" class="mb-2 block text-sm font-medium text-text-primary">
					Address <span class="text-xs text-text-secondary">(search for locations, POIs, or addresses)</span>
				</label>
				<input
					type="text"
					id="address-search"
					value={searchQuery}
					oninput={handleSearchInput}
					onfocus={() => { if (searchResults.length > 0) showResults = true; }}
					class="rounded-base block w-full border-2 border-border-primary bg-bg-primary px-3 py-2 text-text-primary focus:ring-2 focus:outline-none"
					placeholder="e.g. McDonald's London, 123 Regent St, or Tower Bridge"
					autocomplete="off"
				/>
				{#if isSearching}
					<div class="absolute right-3 top-[2.25rem] text-xs text-text-secondary">Searching...</div>
				{/if}
				{#if showResults && searchResults.length > 0}
					<div class="absolute z-50 mt-1 max-h-60 w-full overflow-auto rounded-md border-2 border-border-primary bg-bg-primary shadow-lg">
						{#each searchResults as result}
							<button
								type="button"
								class="w-full px-4 py-2 text-left text-sm text-text-primary hover:bg-bg-secondary focus:bg-bg-secondary focus:outline-none"
								onclick={() => selectLocation(result)}
							>
								<div class="font-medium">{result.display_name.split(',')[0]}</div>
								<div class="text-xs text-text-secondary truncate">{result.display_name}</div>
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
				<div class="mb-2 flex items-center justify-between">
					<h3 class="text-xl font-bold text-text-primary">{branch.name}</h3>
					<span class="text-xs text-text-secondary">ID: {branch.id.slice(0, 8)}...</span>
				</div>
				<p class="text-text-secondary">{branch.address}</p>
				<div class="mt-4 border-t-2 border-border-secondary pt-4">
					<span class="text-xs font-medium text-text-secondary"
						>CREATED AT: {new Date(branch.created_at).toLocaleDateString()}</span
					>
				</div>
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
