<script lang="ts">
	import { api } from '$lib/api';
	import type { PageData } from './$types';

	const { data } = $props<{ data: PageData }>();
	const branches = $derived([...data.branches]);
	let newBranchName = $state('');
	let newBranchAddress = $state('');
	let isSubmitting = $state(false);

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
				branches.push(branch);
				newBranchName = '';
				newBranchAddress = '';
			}
		} catch (error) {
			console.error('Error creating branch:', error);
			alert('An unexpected error occurred');
		} finally {
			isSubmitting = false;
		}
	}
</script>

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
			<div class="flex-1">
				<label for="address" class="mb-2 block text-sm font-medium text-text-primary">Address</label
				>
				<input
					type="text"
					id="address"
					bind:value={newBranchAddress}
					class="rounded-base block w-full border-2 border-border-primary bg-bg-primary px-3 py-2 text-text-primary focus:ring-2 focus:outline-none"
					placeholder="e.g. 123 Regent St"
					required
				/>
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
