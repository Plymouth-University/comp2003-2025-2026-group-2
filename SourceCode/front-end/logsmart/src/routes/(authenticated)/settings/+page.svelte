<script lang="ts">
	import type { PageData, ActionData } from './$types';
	import { enhance } from '$app/forms';
	import { isDarkMode } from '$lib/stores/theme';

	let { data, form } = $props<{ data: PageData; form: ActionData }>();

	let firstName = $state(data.user?.first_name || '');
	let lastName = $state(data.user?.last_name || '');
	let email = $state(data.user?.email || '');
	let isSubmitting = $state(false);
	let showSuccessMessage = $state(false);
	let errorMessage = $state('');

	$effect(() => {
		firstName = data.user?.first_name || '';
		lastName = data.user?.last_name || '';
		email = data.user?.email || '';
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

	function handleToggleDarkMode() {
		isDarkMode.toggle();
	}
</script>

<svelte:head>
	<title>Settings</title>
</svelte:head>
<div class="min-h-screen" style="background-color: var(--bg-secondary);">
	<div class="mx-auto max-w-7xl px-6 py-8">
		<!-- Header -->
		<h1 class="mb-8 text-3xl font-bold" style="color: var(--text-primary);">Settings</h1>

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
			<!-- Profile Settings Section -->
			<div
				class="border-2"
				style="border-color: var(--border-primary); background-color: var(--bg-primary);"
			>
				<div class="border-b-2 px-6 py-4" style="border-color: var(--border-primary);">
					<h2 class="text-xl font-bold" style="color: var(--text-primary);">Profile Information</h2>
				</div>
				<div class="px-6 py-6" style="background-color: var(--bg-primary);">
					<form
						method="POST"
						action="?/updateProfile"
						use:enhance={() => {
							isSubmitting = true;
							return async ({ update }) => {
								await update();
								isSubmitting = false;
							};
						}}
						class="max-w-2xl space-y-4"
					>
						<!-- First Name -->
						<div>
							<label
								for="firstName"
								class="mb-2 block text-sm font-medium"
								style="color: var(--text-primary);"
							>
								First Name
							</label>
							<input
								id="firstName"
								name="firstName"
								type="text"
								bind:value={firstName}
								class="w-full border-2 px-4 py-2 focus:ring-2 focus:outline-none"
								style="border-color: var(--border-primary); background-color: var(--bg-primary); color: var(--text-primary);"
								placeholder="Enter your first name"
								required
							/>
						</div>

						<!-- Last Name -->
						<div>
							<label
								for="lastName"
								class="mb-2 block text-sm font-medium"
								style="color: var(--text-primary);"
							>
								Last Name
							</label>
							<input
								id="lastName"
								name="lastName"
								type="text"
								bind:value={lastName}
								class="w-full border-2 px-4 py-2 focus:ring-2 focus:outline-none"
								style="border-color: var(--border-primary); background-color: var(--bg-primary); color: var(--text-primary);"
								placeholder="Enter your last name"
								required
							/>
						</div>

						<!-- Email -->
						<div>
							<label
								for="email"
								class="mb-2 block text-sm font-medium"
								style="color: var(--text-primary);"
							>
								Email Address
							</label>
							<input
								id="email"
								name="email"
								type="email"
								bind:value={email}
								class="w-full border-2 px-4 py-2 focus:ring-2 focus:outline-none"
								style="border-color: var(--border-secondary); background-color: var(--bg-secondary); color: var(--text-secondary);"
								placeholder="Enter your email"
								readonly
								disabled
								title="Email cannot be changed"
							/>
							<p class="mt-1 text-sm" style="color: var(--text-secondary);">
								Email address cannot be changed
							</p>
						</div>

						<!-- Save Button -->
						<div class="pt-2">
							<button
								type="submit"
								disabled={isSubmitting}
								class="border-2 px-8 py-2 font-medium hover:opacity-80 disabled:cursor-not-allowed disabled:opacity-50"
								style="border-color: var(--border-primary); background-color: var(--bg-primary); color: var(--text-primary);"
							>
								{isSubmitting ? 'Saving...' : 'Save Profile'}
							</button>
						</div>
					</form>
				</div>
			</div>

			<!-- Password Change Section -->
			<div
				class="border-2"
				style="border-color: var(--border-primary); background-color: var(--bg-primary);"
			>
				<div class="border-b-2 px-6 py-4" style="border-color: var(--border-primary);">
					<h2 class="text-xl font-bold" style="color: var(--text-primary);">Reset Password</h2>
				</div>
				<div class="px-6 py-6" style="background-color: var(--bg-primary);">
					<div class="max-w-2xl">
						<p class="mb-4" style="color: var(--text-secondary);">
							To change your password, we'll send a password reset link to your email address.
						</p>
						<form
							method="POST"
							action="?/changePassword"
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
								{isSubmitting ? 'Sending...' : 'Send Password Reset Email'}
							</button>
						</form>
					</div>
				</div>
			</div>

			<!-- Appearance Settings Section -->
			<div
				class="border-2"
				style="border-color: var(--border-primary); background-color: var(--bg-primary);"
			>
				<div class="border-b-2 px-6 py-4" style="border-color: var(--border-primary);">
					<h2 class="text-xl font-bold" style="color: var(--text-primary);">Appearance</h2>
				</div>
				<div class="px-6 py-6" style="background-color: var(--bg-primary);">
					<div class="max-w-2xl">
						<div class="flex items-center justify-between">
							<div>
								<h3 class="font-medium" style="color: var(--text-primary);">Dark Mode</h3>
								<p class="mt-1 text-sm" style="color: var(--text-secondary);">
									Switch between light and dark theme
								</p>
							</div>
							<!-- Toggle Switch -->
							<button
								onclick={handleToggleDarkMode}
								class="relative inline-flex h-8 w-14 items-center rounded-full transition-colors focus:ring-2 focus:ring-offset-2 focus:outline-none"
								style="background-color: {$isDarkMode ? '#94C5CC' : '#E5E5E5'};"
								role="switch"
								aria-checked={$isDarkMode}
								aria-label="Toggle dark mode"
							>
								<span
									class="inline-block h-6 w-6 transform rounded-full bg-white transition-transform"
									style="transform: translateX({$isDarkMode ? '30px' : '4px'});"
								></span>
							</button>
						</div>
						<div class="mt-4 border-t pt-4" style="border-color: var(--border-secondary);">
							<p class="text-sm" style="color: var(--text-secondary);">
								Current theme: <span class="font-medium" style="color: var(--text-primary);"
									>{$isDarkMode ? 'Dark' : 'Light'}</span
								>
							</p>
						</div>
					</div>
				</div>
			</div>
		</div>
	</div>
</div>
