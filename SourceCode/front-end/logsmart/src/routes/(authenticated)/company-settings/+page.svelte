<script lang="ts">
	import type { PageData } from './$types';
	import { invalidateAll } from '$app/navigation';
	import PictureUploader from '$lib/components/PictureUploader.svelte';

	let { data } = $props<{ data: PageData }>();

	let companyName = $derived(data.company?.name || '');
	let companyAddress = $derived(data.company?.address || '');
	let logoUrl = $derived(data.company?.logo_url || null);
	let dataExportedAt = $derived(data.company?.data_exported_at || null);

	let effectivePictureUrl = $derived(logoUrl);

	let isSubmitting = $state(false);
	let showSuccessMessage = $state(false);
	let successMessage = $state('');
	let errorMessage = $state('');

	let addressSearchQuery = $state('');
	let addressSearchResults: Array<{ display_name: string; lat: string; lon: string }> = $state([]);
	let showAddressResults = $state(false);
	let addressSearchTimeout: ReturnType<typeof setTimeout> | null = null;
	let isSearchingAddress = $state(false);

	async function searchAddresses(query: string) {
		if (!query || query.length < 3) {
			addressSearchResults = [];
			showAddressResults = false;
			return;
		}

		isSearchingAddress = true;
		try {
			const response = await fetch(
				`https://nominatim.openstreetmap.org/search?q=${encodeURIComponent(query)}&format=json&limit=5&addressdetails=1`,
				{
					headers: {
						'Accept-Language': 'en'
					}
				}
			);

			if (response.ok) {
				const results = await response.json();
				addressSearchResults = results;
				showAddressResults = results.length > 0;
			}
		} catch (error) {
			console.error('Error searching addresses:', error);
		} finally {
			isSearchingAddress = false;
		}
	}

	function handleAddressInput(event: Event) {
		const target = event.target as HTMLInputElement;
		companyAddress = target.value;
		addressSearchQuery = target.value;

		if (addressSearchTimeout) {
			clearTimeout(addressSearchTimeout);
		}

		addressSearchTimeout = setTimeout(() => {
			searchAddresses(addressSearchQuery);
		}, 500);
	}

	function selectAddress(result: { display_name: string }) {
		companyAddress = result.display_name;
		addressSearchQuery = result.display_name;
		showAddressResults = false;
		addressSearchResults = [];
	}

	function handleClickOutside(event: MouseEvent) {
		const target = event.target as HTMLElement;
		if (!target.closest('.address-search-container')) {
			showAddressResults = false;
		}
	}

	async function handleUpdateCompany(e: Event) {
		e.preventDefault();
		if (!data.company?.id) return;

		isSubmitting = true;
		errorMessage = '';
		showSuccessMessage = false;

		try {
			const res = await fetch(`/api/companies/${data.company.id}`, {
				method: 'PUT',
				headers: {
					'Content-Type': 'application/json'
				},
				body: JSON.stringify({
					name: companyName,
					address: companyAddress
				})
			});

			if (!res.ok) {
				const err = await res.json();
				errorMessage = err.error || 'Failed to update company';
				return;
			}

			successMessage = 'Company details saved successfully!';
			showSuccessMessage = true;
			setTimeout(() => {
				showSuccessMessage = false;
			}, 3000);
			invalidateAll();
		} catch {
			errorMessage = 'Failed to update company';
		} finally {
			isSubmitting = false;
		}
	}

	async function handleExportData() {
		if (!data.company?.id) return;

		isSubmitting = true;
		errorMessage = '';
		showSuccessMessage = false;

		try {
			const res = await fetch(`/api/companies/${data.company.id}/export`, {
				method: 'POST'
			});

			if (!res.ok) {
				const err = await res.json();
				errorMessage = err.error || 'Failed to export data';
				return;
			}

			const result = await res.json();
			successMessage = result.message || 'Data export initiated';
			showSuccessMessage = true;
			setTimeout(() => {
				showSuccessMessage = false;
			}, 3000);
			invalidateAll();
		} catch {
			errorMessage = 'Failed to export data';
		} finally {
			isSubmitting = false;
		}
	}

	async function deleteCompany() {
		if (!data.company?.id) return;
		if (!confirm('Are you sure you want to delete the company?')) return;

		isSubmitting = true;
		errorMessage = '';
		showSuccessMessage = false;

		try {
			const res = await fetch(`/api/companies/${data.company.id}`, {
				method: 'DELETE'
			});

			if (!res.ok) {
				const err = await res.json();
				errorMessage = err.error || 'Failed to delete company';
				return;
			}

			const result = await res.json();
			successMessage = result.message || 'Company deletion requested';
			showSuccessMessage = true;
			setTimeout(() => {
				showSuccessMessage = false;
			}, 3000);
			invalidateAll();
		} catch {
			errorMessage = 'Failed to delete company';
		} finally {
			isSubmitting = false;
		}
	}
</script>

<svelte:window on:click={handleClickOutside} />

<svelte:head>
	<title>Company Settings</title>
</svelte:head>
<div class="h-full w-full bg-bg-secondary">
	<div class="mx-auto max-w-7xl px-6 py-8">
		<!-- Header -->
		<h1 class="mb-8 text-3xl font-bold text-text-primary">Company Settings</h1>

		<!-- Success Message -->
		{#if showSuccessMessage}
			<div class="mb-6 border-2 border-green-500 bg-green-50 px-6 py-4 dark:bg-green-900">
				<p class="font-medium text-green-700 dark:text-green-200">
					✓ {successMessage}
				</p>
			</div>
		{/if}

		<!-- Error Message -->
		{#if errorMessage}
			<div class="mb-6 border-2 border-red-500 bg-red-50 px-6 py-4 dark:bg-red-900">
				<p class="font-medium text-red-700 dark:text-red-200">✗ {errorMessage}</p>
			</div>
		{/if}

		<div class="space-y-6">
			<!-- Company Info Settings Section -->
			<div class="border-2 border-border-primary bg-bg-primary">
				<div class="border-b-2 border-border-primary px-6 py-4">
					<h2 class="text-xl font-bold text-text-primary">Company Information</h2>
				</div>
				<div class="bg-bg-primary px-6 py-6">
					<div class="flex flex-col gap-6 md:flex-row">
						<form onsubmit={handleUpdateCompany} class="max-w-2xl flex-1 space-y-4">
							<!-- Company Name -->
							<div>
								<label for="companyName" class="mb-2 block text-sm font-medium text-text-primary">
									Company Name
								</label>
								<input
									id="companyName"
									name="companyName"
									type="text"
									bind:value={companyName}
									oninput={() => {
										// Force reactivity update
									}}
									class="w-full border-2 border-border-primary bg-bg-primary px-4 py-2 text-text-primary focus:ring-2 focus:outline-none"
									placeholder="Enter the company's name"
									required
								/>
							</div>

							<!-- Company Address -->
							<div class="address-search-container relative">
								<label
									for="companyAddress"
									class="mb-2 block text-sm font-medium text-text-primary"
								>
									Company Headquarters Address
								</label>
								<input
									id="companyAddress"
									name="companyAddress"
									type="text"
									bind:value={companyAddress}
									oninput={handleAddressInput}
									class="w-full border-2 border-border-primary bg-bg-primary px-4 py-2 text-text-primary focus:ring-2 focus:outline-none"
									placeholder="Search for address..."
									autocomplete="off"
									required
								/>
								{#if isSearchingAddress}
									<div
										class="absolute z-10 w-full rounded-md border border-gray-300 bg-white p-2 text-center dark:bg-gray-800"
									>
										<span class="text-sm text-gray-500">Searching...</span>
									</div>
								{:else if showAddressResults && addressSearchResults.length > 0}
									<ul
										class="absolute z-10 max-h-60 w-full overflow-auto rounded-md border border-gray-300 bg-white shadow-lg dark:bg-gray-800"
									>
										{#each addressSearchResults as result (result.lat + result.lon)}
											<li>
												<button
													type="button"
													class="w-full cursor-pointer px-4 py-2 text-left text-sm text-text-primary hover:bg-gray-100 dark:hover:bg-gray-700"
													onclick={() => selectAddress(result)}
												>
													{result.display_name}
												</button>
											</li>
										{/each}
									</ul>
								{/if}
								<p class="mt-1 text-xs text-text-secondary">
									>(search for locations, POIs, or addresses)
								</p>
							</div>

							<!-- Save Button -->
							<div class="pt-2">
								<button
									type="submit"
									disabled={isSubmitting ||
										!companyName.trim() ||
										!companyAddress.trim() ||
										(companyName === data.company?.name &&
											companyAddress === data.company?.address)}
									class="border-2 border-border-primary bg-bg-primary px-8 py-2 font-medium text-text-primary hover:opacity-80 disabled:cursor-not-allowed disabled:opacity-50"
								>
									{isSubmitting ? 'Saving...' : 'Save Company Details'}
								</button>
							</div>
						</form>

						<!-- Company Logo -->
						<div class="flex flex-1 flex-col items-center justify-start pt-4 md:pt-0 md:pl-8">
							<PictureUploader
								type="company_logo"
								companyId={data.company?.id}
								currentPictureUrl={effectivePictureUrl}
								onUploadComplete={() => {
									invalidateAll();
								}}
								onDeleteComplete={() => {
									invalidateAll();
								}}
							/>
						</div>
					</div>
				</div>
			</div>

			<!-- Company Data Management -->
			<div class="border-2 border-border-primary bg-bg-primary">
				<div class="border-b-2 border-border-primary px-6 py-4">
					<h2 class="text-xl font-bold text-text-primary">Export Company Data</h2>
				</div>
				<div class="bg-bg-primary px-6 py-6">
					<div class="max-w-2xl">
						<p class="mb-4 text-text-secondary">
							Data will be sent to the email address used to register your company.
							{#if dataExportedAt}
								<span class="text-green-600"
									>Data exported on {new Date(dataExportedAt).toLocaleDateString()}</span
								>
							{/if}
						</p>
						<button
							onclick={handleExportData}
							disabled={isSubmitting}
							class="border-2 border-border-primary bg-bg-primary px-8 py-2 font-medium text-text-primary hover:opacity-80 disabled:cursor-not-allowed disabled:opacity-50"
						>
							{isSubmitting
								? 'Exporting...'
								: dataExportedAt
									? 'Re-export Company Data'
									: 'Export Company Data'}
						</button>
					</div>
				</div>
			</div>

			<!-- Delete Company -->
			<div class="border-2 border-border-primary bg-bg-primary">
				<div class="border-b-2 border-border-primary px-6 py-4">
					<h2 class="text-xl font-bold text-button-secondary">Delete Company</h2>
				</div>
				<div class="bg-bg-primary px-6 py-6">
					<div class="max-w-2xl">
						<p class="mb-4 text-text-secondary">
							Company data must be exported prior to deletion. Data is retained on our servers for
							30 days thereafter.
						</p>
					</div>
					<button
						onclick={deleteCompany}
						disabled={!dataExportedAt || isSubmitting}
						class="cursor-pointer border-2 border-solid px-8 py-2 text-red-500 hover:text-red-700 disabled:cursor-not-allowed disabled:opacity-50"
						aria-label="Delete company"
						title={!dataExportedAt ? 'You must export company data before requesting deletion' : ''}
					>
						Delete Company
					</button>
					{#if !dataExportedAt}
						<p class="mt-2 text-sm text-text-secondary">Export company data to enable deletion</p>
					{/if}
				</div>
			</div>
		</div>
	</div>
</div>
