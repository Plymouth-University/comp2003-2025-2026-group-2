<script lang="ts">
	import type { PageData, ActionData } from './$types';
	import { enhance } from '$app/forms';
	import { isDarkMode } from '$lib/stores/theme';
	import { startRegistration } from '@simplewebauthn/browser';
	import { invalidateAll } from '$app/navigation';
	import { onMount } from 'svelte';
	import ProfilePictureUploader from '$lib/components/PictureUploader.svelte';

	let { data, form } = $props<{ data: PageData; form: ActionData }>();

	let firstName = $derived(data.user?.first_name || '');
	let lastName = $derived(data.user?.last_name || '');
	let email = $derived(data.user?.email || '');
	let hasGoogleLinked = $derived(data.user?.oauth_provider === 'google');
	let profilePictureUrl = $derived(data.user?.profile_picture_url || null);
	let oauthPictureUrl = $derived(data.user?.oauth_picture || null);

	let effectivePictureUrl = $derived(profilePictureUrl ? profilePictureUrl : oauthPictureUrl);

	let isSubmitting = $state(false);
	let showSuccessMessage = $state(false);
	let errorMessage = $state('');
	let passkeyName = $state('');
	let isRegisteringPasskey = $state(false);
	let isLinkingGoogle = $state(false);
	let showLinkSuccessMessage = $state(false);

	onMount(() => {
		const urlParams = new URLSearchParams(window.location.search);
		const oauthLinkToken = urlParams.get('oauth_link_token');

		if (oauthLinkToken) {
			confirmGoogleLink(oauthLinkToken).then(() => {
				const url = new URL(window.location.href);
				url.searchParams.delete('oauth_link_token');
				window.history.replaceState({}, '', url);
			});
		}
	});

	async function confirmGoogleLink(linkToken: string) {
		try {
			const resp = await fetch('/api/auth/google/link/confirm', {
				method: 'POST',
				headers: { 'Content-Type': 'application/json' },
				body: JSON.stringify({
					link_token: linkToken
				})
			});

			if (!resp.ok) {
				const err = await resp.json();
				throw new Error(err.error || 'Failed to link Google account');
			}

			showLinkSuccessMessage = true;
			await invalidateAll();
			setTimeout(() => {
				showLinkSuccessMessage = false;
			}, 3000);
		} catch (e: unknown) {
			console.error(e);
			const message = e instanceof Error ? e.message : 'Failed to link Google account';
			errorMessage = message || 'Failed to link Google account';
		}
	}

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

			const attResp = await startRegistration({ optionsJSON: startData.options });

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
			await invalidateAll();
		} catch (e: unknown) {
			console.error(e);
			const message = e instanceof Error ? e.message : 'Failed to register passkey';
			errorMessage = message || 'Failed to register passkey';
		} finally {
			isRegisteringPasskey = false;
		}
	}

	async function handleLinkGoogle() {
		isLinkingGoogle = true;
		window.location.href = '/api/auth/google/initiate?mode=link';
	}

	async function handleUnlinkGoogle() {
		if (!confirm('Are you sure you want to unlink your Google account?')) return;
		try {
			const resp = await fetch('/api/auth/google/unlink', { method: 'DELETE' });
			if (resp.ok) {
				await invalidateAll();
				showSuccessMessage = true;
				setTimeout(() => {
					showSuccessMessage = false;
				}, 3000);
			} else {
				const err = await resp.json();
				errorMessage = err.error || 'Failed to unlink Google account';
			}
		} catch {
			errorMessage = 'Failed to unlink Google account';
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
		} catch {
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
<div class="settings-page min-h-full bg-bg-secondary">
	<div class="mx-auto max-w-7xl px-6 py-8">
		<!-- Header -->
		<h1 class="mb-8 text-3xl font-bold text-text-primary">Settings</h1>

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
			<div class="border-2 border-border-primary bg-bg-primary">
				<div class="border-b-2 border-border-primary px-6 py-4">
					<h2 class="text-xl font-bold text-text-primary">Profile Information</h2>
				</div>
				<div class="bg-bg-primary px-6 py-6">
					<div class="flex flex-col gap-6 md:flex-row">
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
							class="max-w-2xl flex-1 space-y-4"
						>
							<!-- First Name -->
							<div>
								<label for="firstName" class="mb-2 block text-sm font-medium text-text-primary">
									First Name
								</label>
								<input
									id="firstName"
									name="firstName"
									type="text"
									bind:value={firstName}
									class="w-full border-2 border-border-primary bg-bg-primary px-4 py-2 text-text-primary focus:ring-2 focus:outline-none"
									placeholder="Enter your first name"
									required
								/>
							</div>

							<!-- Last Name -->
							<div>
								<label for="lastName" class="mb-2 block text-sm font-medium text-text-primary">
									Last Name
								</label>
								<input
									id="lastName"
									name="lastName"
									type="text"
									bind:value={lastName}
									class="w-full border-2 border-border-primary bg-bg-primary px-4 py-2 text-text-primary focus:ring-2 focus:outline-none"
									placeholder="Enter your last name"
									required
								/>
							</div>

							<!-- Email -->
							<div>
								<label for="email" class="mb-2 block text-sm font-medium text-text-primary">
									Email Address
								</label>
								<input
									id="email"
									name="email"
									type="email"
									bind:value={email}
									class="w-full border-2 border-border-secondary bg-bg-secondary px-4 py-2 text-text-secondary focus:ring-2 focus:outline-none"
									placeholder="Enter your email"
									readonly
									disabled
									title="Email cannot be changed"
								/>
								<p class="mt-1 text-sm text-text-secondary">Email address cannot be changed</p>
							</div>

							<!-- Save Button -->
							<div class="pt-2">
								<button
									type="submit"
									disabled={isSubmitting ||
										!firstName.trim() ||
										!lastName.trim() ||
										(firstName === data.user?.first_name && lastName === data.user?.last_name)}
									class="border-2 border-border-primary bg-bg-primary px-8 py-2 font-medium text-text-primary hover:opacity-80 disabled:cursor-not-allowed disabled:opacity-50"
								>
									{isSubmitting ? 'Saving...' : 'Save Profile'}
								</button>
							</div>
						</form>

						<!-- Profile Picture on the right side -->
						<div class="flex flex-1 flex-col items-center justify-start pt-4 md:pt-0 md:pl-8">
							<ProfilePictureUploader
								type="pfp"
								currentPictureUrl={effectivePictureUrl}
								{firstName}
								{lastName}
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

			<!-- Password Change Section -->
			<div class="border-2 border-border-primary bg-bg-primary">
				<div class="border-b-2 border-border-primary px-6 py-4">
					<h2 class="text-xl font-bold text-text-primary">Reset Password</h2>
				</div>
				<div class="bg-bg-primary px-6 py-6">
					<div class="max-w-2xl">
						<p class="mb-4 text-text-secondary">
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
								class="border-2 border-border-primary bg-bg-primary px-8 py-2 font-medium text-text-primary hover:opacity-80 disabled:cursor-not-allowed disabled:opacity-50"
							>
								{isSubmitting ? 'Sending...' : 'Send Password Reset Email'}
							</button>
						</form>
					</div>
				</div>
			</div>

			<!-- Passkeys Section -->
			<div class="border-2 border-border-primary bg-bg-primary">
				<div class="border-b-2 border-border-primary px-6 py-4">
					<h2 class="text-xl font-bold text-text-primary">Passkeys</h2>
				</div>
				<div class="bg-bg-primary px-6 py-6">
					<div class="max-w-2xl">
						<p class="mb-4 text-text-secondary">
							Passkeys allow you to sign in safely without a password using your fingerprint, face,
							or hardware key.
						</p>

						<!-- List Existing Passkeys -->
						{#if data.passkeys && data.passkeys.length > 0}
							<div class="mb-6 space-y-3">
								{#each data.passkeys as pk (pk.id)}
									<div
										class="flex items-center justify-between rounded border-2 border-border-secondary p-3"
									>
										<div>
											<p class="font-medium text-text-primary">{pk.name}</p>
											<p class="text-xs text-text-secondary">
												Added on {new Date(pk.created_at).toLocaleDateString()}
											</p>
										</div>
										<button
											onclick={() => deletePasskey(pk.id)}
											class="cursor-pointer text-red-500 hover:text-red-700"
											aria-label="Delete passkey"
										>
											Delete
										</button>
									</div>
								{/each}
							</div>
						{/if}

						<div class="flex items-end gap-4">
							<div class="grow">
								<label for="passkeyName" class="mb-2 block text-sm font-medium text-text-primary">
									Passkey Name (Optional)
								</label>
								<input
									id="passkeyName"
									type="text"
									bind:value={passkeyName}
									class="w-full border-2 border-border-primary bg-bg-primary px-4 py-2 text-text-primary focus:ring-2 focus:outline-none"
									placeholder="e.g. My MacBook"
								/>
							</div>
							<button
								onclick={handleRegisterPasskey}
								disabled={isRegisteringPasskey}
								class="border-2 border-border-primary bg-bg-primary px-6 py-2 font-medium text-text-primary hover:opacity-80 disabled:cursor-not-allowed disabled:opacity-50"
							>
								{isRegisteringPasskey ? 'Registering...' : 'Add Passkey'}
							</button>
						</div>
					</div>
				</div>
			</div>

			<!-- Google Account Linking Section -->
			<div class="border-2 border-border-primary bg-bg-primary">
				<div class="border-b-2 border-border-primary px-6 py-4">
					<h2 class="text-xl font-bold text-text-primary">Linked Accounts</h2>
				</div>
				<div class="bg-bg-primary px-6 py-6">
					<div class="max-w-2xl">
						<p class="mb-4 text-text-secondary">
							Link your Google account to sign in with one click.
						</p>

						{#if hasGoogleLinked}
							<div
								class="mb-4 rounded border-2 border-green-500 bg-green-50 px-4 py-3 dark:bg-green-900"
							>
								<p class="font-medium text-green-700 dark:text-green-200">
									✓ Google account is linked
								</p>
							</div>
							<button
								onclick={handleUnlinkGoogle}
								class="cursor-pointer border-2 border-border-primary bg-bg-primary px-8 py-2 font-medium text-text-primary hover:opacity-80"
							>
								Unlink Google Account
							</button>
						{:else if showLinkSuccessMessage}
							<div
								class="mb-4 rounded border-2 border-green-500 bg-green-50 px-4 py-3 dark:bg-green-900"
							>
								<p class="font-medium text-green-700 dark:text-green-200">
									✓ Google account linked successfully!
								</p>
							</div>
						{/if}

						{#if !hasGoogleLinked}
							<button
								onclick={handleLinkGoogle}
								disabled={isLinkingGoogle}
								class="border-2 border-border-primary bg-bg-primary px-8 py-2 font-medium text-text-primary hover:opacity-80 disabled:cursor-not-allowed disabled:opacity-50"
							>
								{isLinkingGoogle ? 'Linking...' : 'Link Google Account'}
							</button>
						{/if}
					</div>
				</div>
			</div>

			<!-- Appearance Settings Section -->
			<div class="border-2 border-border-primary bg-bg-primary">
				<div class="border-b-2 border-border-primary px-6 py-4">
					<h2 class="text-xl font-bold text-text-primary">Appearance</h2>
				</div>
				<div class="bg-bg-primary px-6 py-6">
					<div class="max-w-2xl">
						<div class="flex items-center justify-between">
							<div>
								<h3 class="font-medium text-text-primary">Dark Mode</h3>
								<p class="mt-1 text-sm text-text-secondary">Switch between light and dark theme</p>
							</div>
							<!-- Toggle Switch -->
							<button
								onclick={handleToggleDarkMode}
								class="relative inline-flex h-8 w-14 items-center rounded-full transition-colors focus:ring-2 focus:ring-offset-2 focus:outline-none"
								style="background-color: {$isDarkMode ? 'var(--button-primary)' : '#E5E5E5'};"
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
						<div class="mt-4 border-t border-border-secondary pt-4">
							<p class="text-sm text-text-secondary">
								Current theme: <span class="font-medium text-text-primary"
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

<style>
	.settings-page button:not(:disabled) {
		cursor: pointer;
		transition:
			transform 0.12s ease,
			filter 0.12s ease;
	}

	.settings-page button:not(:disabled):hover {
		transform: translateY(-1px) scale(1.02);
		filter: brightness(0.96);
	}

	.settings-page button:disabled {
		cursor: not-allowed;
	}
</style>
