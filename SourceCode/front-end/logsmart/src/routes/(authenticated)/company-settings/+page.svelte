<script lang="ts">
	import type { PageData, ActionData } from './$types';
	import { enhance } from '$app/forms';
	import { startRegistration } from '@simplewebauthn/browser';
	import { invalidateAll } from '$app/navigation';
	import { onMount } from 'svelte';
	import ProfilePictureUploader from '$lib/components/ProfilePictureUploader.svelte';

	let { data, form } = $props<{ data: PageData; form: ActionData }>();

	let companyName = $derived(data.company?.company_name || '');
	let companyAddress = $derived(data.company?.company_address || '');
	let email = $derived(data.user?.email || '');
	let logoUrl = $derived(data.company?.company_logo_url || null);

	let effectivePictureUrl = $derived(logoUrl);

	let isSubmitting = $state(false);
	let showSuccessMessage = $state(false);
	let errorMessage = $state('');

	async function deleteCompany() {
		if (!confirm('Are you sure you want to delete the company?'))
			try {
				const resp = await fetch('/api/auth/');
				if (resp.ok) {
					showSuccessMessage = true;
					setTimeout(() => {
						showSuccessMessage = false;
					}, 3000);
				} else {
					const err = await resp.json();
					errorMessage = err.error || 'Failed to delete company.';
				}
			} catch {
				errorMessage = 'Failed to delete company.';
			}
	}

	$effect(() => {
		companyName = data.user?.company_name || '';
		companyAddress = data.company?.company_address || '';
	});

	$effect(() => {
		if (form?.success) {
			showSuccessMessage = true;
			errorMessage = '';

			setTimeout(() => {
				showSuccessMessage = false;
			}, 3000);
		} else if (form?.message) {
			errorMessage = form.message;
			console.error('Form error:', form);
			setTimeout(() => {
				errorMessage = '';
			}, 5000);
		}
	});
</script>

<svelte:head>
	<title>Company Settings</title>
</svelte:head>
<div class="h-full w-full" style="background-color: var(--bg-secondary);">
	<div class="mx-auto max-w-7xl px-6 py-8">
		<!-- Header -->
		<h1 class="mb-8 text-3xl font-bold" style="color: var(--text-primary);">Company Settings</h1>

		<!-- Success Message -->
		{#if showSuccessMessage}
			<div class="mb-6 border-2 border-green-500 bg-green-50 px-6 py-4 dark:bg-green-900">
				<p class="font-medium text-green-700 dark:text-green-200">
					✓ {form?.message || 'Changes saved successfully!'}
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
			<div
				class="border-2"
				style="border-color: var(--border-primary); background-color: var(--bg-primary);"
			>
				<div class="border-b-2 px-6 py-4" style="border-color: var(--border-primary);">
					<h2 class="text-xl font-bold" style="color: var(--text-primary);">Company Information</h2>
				</div>
				<div class="px-6 py-6" style="background-color: var(--bg-primary);">
					<div class="flex flex-col gap-6 md:flex-row">
						<form
							method="POST"
							action="?/updateCompany"
							use:enhance={() => {
								isSubmitting = true;
								return async ({ update }) => {
									await update();
									isSubmitting = false;
								};
							}}
							class="max-w-2xl flex-1 space-y-4"
						>
							<!-- Company Name -->
							<div>
								<label
									for="companyName"
									class="mb-2 block text-sm font-medium"
									style="color: var(--text-primary);"
								>
									Company Name
								</label>
								<input
									id="companyName"
									name="companyName"
									type="text"
									bind:value={companyName}
									class="w-full border-2 px-4 py-2 focus:ring-2 focus:outline-none"
									style="border-color: var(--border-primary); background-color: var(--bg-primary); color: var(--text-primary);"
									placeholder="Enter the company's name"
									required
								/>
							</div>

							<!-- Company Address -->
							<div>
								<label
									for="companyAddress"
									class="mb-2 block text-sm font-medium"
									style="color: var(--text-primary);"
								>
									Company Headquarters Address
								</label>
								<input
									id="companyAddress"
									name="companyAddress"
									type="text"
									bind:value={companyAddress}
									class="w-full border-2 px-4 py-2 focus:ring-2 focus:outline-none"
									style="border-color: var(--border-primary); background-color: var(--bg-primary); color: var(--text-primary);"
									placeholder="Enter the address of headquarters"
									required
								/>
							</div>

							<!-- Save Button -->
							<div class="pt-2">
								<button
									type="submit"
									disabled={isSubmitting ||
										!companyName.trim() ||
										!companyAddress.trim() ||
										(companyName == data.user?.company_name &&
											companyName == data.company?.company_name &&
											companyAddress == data.company?.company_address)}
									class="border-2 px-8 py-2 font-medium hover:opacity-80 disabled:cursor-not-allowed disabled:opacity-50"
									style="border-color: var(--border-primary); background-color: var(--bg-primary); color: var(--text-primary);"
								>
									{isSubmitting ? 'Saving...' : 'Save Company Details'}
								</button>
							</div>
						</form>

						<!-- Company Logo -->
						<div class="flex flex-1 flex-col items-center justify-start pt-4 md:pt-0 md:pl-8">
							<ProfilePictureUploader
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
			<div
				class="border-2"
				style="border-color: var(--border-primary); background-color: var(--bg-primary);"
			>
				<div class="border-b-2 px-6 py-4" style="border-color: var(--border-primary);">
					<h2 class="text-xl font-bold" style="color: var(--text-primary);">Export Company Data</h2>
				</div>
				<div class="px-6 py-6" style="background-color: var(--bg-primary);">
					<div class="max-w-2xl">
						<p class="mb-4" style="color: var(--text-secondary);">
							Data will be sent to the email address used to register your company.
						</p>
						<form
							method="POST"
							action="?/exportData"
							use:enhance={() => {
								isSubmitting = true;
								return async ({ update }) => {
									await update();
									isSubmitting = false;
								};
							}}
						>
							<input type="hidden" name="email" value={email} />
							<button
								type="submit"
								disabled={isSubmitting}
								class="border-2 px-8 py-2 font-medium hover:opacity-80 disabled:cursor-not-allowed disabled:opacity-50"
								style="border-color: var(--border-primary); background-color: var(--bg-primary); color: var(--text-primary);"
							>
								{isSubmitting ? 'Exporting...' : 'Export Company Data'}
							</button>
						</form>
					</div>
				</div>
			</div>

			<!-- Delete Company -->
			<div
				class="border-2"
				style="border-color: var(--border-primary); background-color: var(--bg-primary);"
			>
				<div class="border-b-2 px-6 py-4" style="border-color: var(--border-primary);">
					<h2 class="text-xl font-bold" style="color:#ef4444;">Delete Company</h2>
				</div>
				<div class="px-6 py-6" style="background-color: var(--bg-primary);">
					<div class="max-w-2xl">
						<p class="mb-4" style="color: var(--text-secondary);">
							Company data must be exported prior to deletion. Data is retained on our servers for
							30 days thereafter.
						</p>
					</div>
					<button
						onclick={deleteCompany}
						class="cursor-pointer border-2 border-solid px-8 py-2 text-red-500 hover:text-red-700"
						style="2px solid var(--border-primary);"
						aria-label="Delete company"
					>
						Delete Company
					</button>
				</div>
			</div>
		</div>
	</div>
</div>
