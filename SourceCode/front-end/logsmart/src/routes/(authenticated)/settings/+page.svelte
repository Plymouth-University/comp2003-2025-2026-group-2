<script lang="ts">
	import type { PageData, ActionData } from './$types';
	import { enhance } from '$app/forms';
	import { isDarkMode } from '$lib/stores/theme';
	import { startRegistration } from '@simplewebauthn/browser';
	import { invalidateAll } from '$app/navigation';

	let { data, form } = $props<{ data: PageData; form: ActionData }>();

	let firstName = $derived(data.user?.first_name || '');
	let lastName = $derived(data.user?.last_name || '');
	let email = $derived(data.user?.email || '');
	let isSubmitting = $state(false);
	let showSuccessMessage = $state(false);
	let errorMessage = $state('');
	let passkeyName = $state('');
	let isRegisteringPasskey = $state(false);

	async function handleRegisterPasskey() {
		try {
			isRegisteringPasskey = true;
			const startResp = await fetch('/api/auth/passkey/register/start', {
				method: 'POST',
				headers: { 'Content-Type': 'application/json' },
				body: JSON.stringify({ name: passkeyName || `${firstName}'s Passkey` })
			});

			if (!startResp.ok) throw new Error('Failed to start registration');
			const startData = await startResp.json();

			const attResp = await startRegistration(startData.options);

			const finishResp = await fetch('/api/auth/passkey/register/finish', {
				method: 'POST',
				headers: { 'Content-Type': 'application/json' },
				body: JSON.stringify({
					credential: attResp,
					auth_id: `${startData.auth_id}|${passkeyName || firstName + "'s Passkey"}`
				})
			});

			if (!finishResp.ok) {
				const err = await finishResp.json();
				throw new Error(err.message || err.error || 'Failed to finish registration');
			}

			showSuccessMessage = true;
			passkeyName = '';
			await invalidateAll(); // Reloads data including passkeys
		} catch (e: any) {
			console.error(e);
			errorMessage = e.message || 'Failed to register passkey';
		} finally {
			isRegisteringPasskey = false;
		}
	}

	async function deletePasskey(id: string) {
		if (!confirm('Are you sure you want to remove this passkey?')) return;
		try {
			const resp = await fetch(`/api/auth/passkeys/${id}`, { method: 'DELETE' });
			if (resp.ok) {
				await invalidateAll();
			} else {
				errorMessage = 'Failed to delete passkey';
			}
		} catch (e) {
			errorMessage = 'Failed to delete passkey';
		}
	}

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
								disabled={isSubmitting ||
									!firstName.trim() ||
									!lastName.trim() ||
									(firstName === data.user?.first_name && lastName === data.user?.last_name)}
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

			<!-- Passkeys Section -->
			<div
				class="border-2"
				style="border-color: var(--border-primary); background-color: var(--bg-primary);"
			>
				<div class="border-b-2 px-6 py-4" style="border-color: var(--border-primary);">
					<h2 class="text-xl font-bold" style="color: var(--text-primary);">Passkeys</h2>
				</div>
				<div class="px-6 py-6" style="background-color: var(--bg-primary);">
					<div class="max-w-2xl">
						<p class="mb-4" style="color: var(--text-secondary);">
							Passkeys allow you to sign in safely without a password using your fingerprint, face,
							or hardware key.
						</p>

						<!-- List Existing Passkeys -->
						{#if data.passkeys && data.passkeys.length > 0}
							<div class="mb-6 space-y-3">
								{#each data.passkeys as pk}
									<div
										class="flex items-center justify-between rounded border-2 p-3"
										style="border-color: var(--border-secondary);"
									>
										<div>
											<p class="font-medium" style="color: var(--text-primary);">{pk.name}</p>
											<p class="text-xs" style="color: var(--text-secondary);">
												Added on {new Date(pk.created_at).toLocaleDateString()}
											</p>
										</div>
										<button
											onclick={() => deletePasskey(pk.id)}
											class="text-red-500 hover:text-red-700"
											aria-label="Delete passkey"
										>
											Delete
										</button>
									</div>
								{/each}
							</div>
						{/if}

						<div class="flex items-end gap-4">
							<div class="flex-grow">
								<label
									for="passkeyName"
									class="mb-2 block text-sm font-medium"
									style="color: var(--text-primary);"
								>
									Passkey Name (Optional)
								</label>
								<input
									id="passkeyName"
									type="text"
									bind:value={passkeyName}
									class="w-full border-2 px-4 py-2 focus:ring-2 focus:outline-none"
									style="border-color: var(--border-primary); background-color: var(--bg-primary); color: var(--text-primary);"
									placeholder="e.g. My MacBook"
								/>
							</div>
							<button
								onclick={handleRegisterPasskey}
								disabled={isRegisteringPasskey}
								class="border-2 px-6 py-2 font-medium hover:opacity-80 disabled:cursor-not-allowed disabled:opacity-50"
								style="border-color: var(--border-primary); background-color: var(--bg-primary); color: var(--text-primary);"
							>
								{isRegisteringPasskey ? 'Registering...' : 'Add Passkey'}
							</button>
						</div>
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
								style="background-color: {$isDarkMode ? '#3D7A82' : '#E5E5E5'};"
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
