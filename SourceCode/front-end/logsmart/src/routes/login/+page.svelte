<script lang="ts">
	import { goto, invalidateAll } from '$app/navigation';
	import { startAuthentication } from '@simplewebauthn/browser';
	import { api } from '$lib/api';

	import { onMount } from 'svelte';
	import { browser } from '$app/environment';

	let email = $state('');
	let password = $state('');
	let loading = $state(false);
	let error = $state('');
	let touched = $state({ email: false, password: false });
	const emailValid = $derived(/^\S+@\S+\.\S+$/.test(email));
	const passwordValid = $derived(password.length >= 6);
	const formValid = $derived(emailValid && passwordValid);

	onMount(async () => {
		if (
			browser &&
			window.PublicKeyCredential &&
			window.PublicKeyCredential.isConditionalMediationAvailable &&
			(await window.PublicKeyCredential.isConditionalMediationAvailable())
		) {
			try {
				const startResp = await fetch('/api/auth/passkey/login/discoverable/start', {
					method: 'POST',
					headers: { 'Content-Type': 'application/json' },
					body: JSON.stringify({})
				});

				if (startResp.ok) {
					const startData = await startResp.json();
					// Enable conditional UI
					// @ts-ignore - SimpleWebAuthn v10+ supports 2nd arg for conditional UI
					const authResp = await startAuthentication(startData.options, true);

					// If we get here, the user selected a credential from the autofill
					loading = true;
					const finishResp = await fetch('/api/auth/passkey/login/discoverable/finish', {
						method: 'POST',
						headers: { 'Content-Type': 'application/json' },
						body: JSON.stringify({
							credential: authResp,
							auth_id: startData.auth_id
						})
					});

					if (!finishResp.ok) {
						throw new Error('Authentication failed');
					}

					await invalidateAll();
					await goto('/dashboard');
				}
			} catch (e: any) {
				// Ignore errors from conditional UI (timeout, cancellation, etc)
				console.debug('Conditional UI:', e);
			}
		}
	});

	async function submit(e: Event) {
		e.preventDefault();
		error = '';
		touched = { email: true, password: true };
		if (!formValid) return;
		loading = true;
		try {
			const { data, error: apiError } = await api.POST('/auth/login', {
				body: { email, password }
			});

			if (apiError) {
				error = apiError.error || 'Login failed';
			} else {
				await invalidateAll();
				await goto('/dashboard');
			}
		} catch (err) {
			console.error('Login error:', err);
			error = 'Network error';
		} finally {
			loading = false;
		}
	}

	async function handlePasskeyLogin() {
		loading = true;
		error = '';

		try {
			let startResp;
			let isDiscoverable = !emailValid;

			if (isDiscoverable) {
				// One-click discoverable flow - no email required
				startResp = await fetch('/api/auth/passkey/login/discoverable/start', {
					method: 'POST',
					headers: { 'Content-Type': 'application/json' },
					body: JSON.stringify({})
				});
			} else {
				// Email-first flow (backward compatibility)
				startResp = await fetch('/api/auth/passkey/login/start', {
					method: 'POST',
					headers: { 'Content-Type': 'application/json' },
					body: JSON.stringify({ email })
				});
			}

			if (!startResp.ok) {
				const err = await startResp.json();
				throw new Error(err.error || 'Failed to start passkey login');
			}
			const startData = await startResp.json();

			const authResp = await startAuthentication(startData.options);

			const finishEndpoint = isDiscoverable
				? '/api/auth/passkey/login/discoverable/finish'
				: '/api/auth/passkey/login/finish';

			const finishResp = await fetch(finishEndpoint, {
				method: 'POST',
				headers: { 'Content-Type': 'application/json' },
				body: JSON.stringify({
					credential: authResp,
					auth_id: startData.auth_id
				})
			});

			if (!finishResp.ok) {
				const err = await finishResp.json();
				throw new Error(err.error || 'Authentication failed');
			}

			await invalidateAll();
			await goto('/dashboard');
		} catch (e: any) {
			console.error(e);
			error = e.message || 'Passkey login failed';
		} finally {
			loading = false;
		}
	}
</script>

<svelte:head>
	<title>Log In</title>
</svelte:head>
<div class="flex h-full w-full items-center justify-center bg-bg-primary p-6">
	<form onsubmit={submit} class="form bg-bg-secondary">
		<h1 class="font-semibold text-text-primary">Log in</h1>
		{#if error}
			<div class="error" role="alert">{error}</div>
		{/if}
		<label class="field">
			<span class="label-text text-text-primary">Email</span>
			<input
				type="email"
				class="text-text-primary"
				bind:value={email}
				onblur={() => (touched.email = true)}
				aria-invalid={!emailValid}
				aria-describedby="email-help"
				autocomplete="username webauthn"
				required
			/>
			{#if touched.email && !emailValid}
				<div id="email-help" class="field-error">Enter a valid email address.</div>
			{/if}
		</label>

		<label class="field">
			<span class="label-text text-text-primary">Password</span>
			<input
				type="password"
				class="text-text-primary"
				bind:value={password}
				onblur={() => (touched.password = true)}
				aria-invalid={!passwordValid}
				aria-describedby="password-help"
				required
			/>
			{#if touched.password && !passwordValid}
				<div id="password-help" class="field-error">Password must be at least 6 characters.</div>
			{/if}
		</label>

		<button type="submit" class="btn" disabled={!formValid || loading}>
			{#if loading}
				Signing in...
			{:else}
				Sign in
			{/if}
		</button>

		<div class="mt-4 flex items-center justify-between">
			<hr class="w-full border-gray-300" />
			<span class="px-2 text-sm text-gray-500">OR</span>
			<hr class="w-full border-gray-300" />
		</div>

		<button
			type="button"
			class="btn mt-4"
			style="background-color: #4285F4;"
			onclick={handlePasskeyLogin}
			disabled={loading}
		>
			{#if loading}
				Signing in...
			{:else}
				Sign in with Passkey
			{/if}
		</button>
		<a href="/reset-password" class="mt-4 inline-block text-sm text-indigo-600 hover:underline"
			>Forgot password?</a
		>
	</form>
</div>

<style>
	:global(body) {
		font-family:
			system-ui,
			-apple-system,
			'Segoe UI',
			Roboto,
			'Helvetica Neue',
			Arial;
		margin: 0;
		padding: 0;
	}
	.form {
		width: 100%;
		max-width: 400px;
		padding: 2rem;
		border-radius: 8px;
		box-shadow: 0 6px 20px rgba(20, 20, 50, 0.08);
	}
	h1 {
		margin: 0 0 1rem 0;
		font-size: 1.25rem;
	}
	.field {
		display: flex;
		flex-direction: column;
		margin-bottom: 1rem;
	}
	.label-text {
		font-size: 0.9rem;
		margin-bottom: 0.25rem;
	}
	input {
		padding: 0.5rem 0.75rem;
		border: 1px solid #d1d7e0;
		border-radius: 6px;
		font-size: 1rem;
		outline: none;
	}
	input:focus {
		border-color: #6b8cff;
		box-shadow: 0 0 0 3px rgba(107, 140, 255, 0.12);
	}
	.field-error {
		color: #c93838;
		font-size: 0.875rem;
		margin-top: 0.5rem;
	}
	.error {
		background: #fdecea;
		color: #821313;
		padding: 0.5rem 0.75rem;
		border-radius: 6px;
		margin-bottom: 1rem;
		font-size: 0.95rem;
	}
	.btn {
		width: 100%;
		padding: 0.75rem 1rem;
		background: #2f6fff;
		color: white;
		border: none;
		border-radius: 6px;
		font-size: 1rem;
		cursor: pointer;
	}
	.btn[disabled] {
		opacity: 0.6;
		cursor: not-allowed;
	}
</style>
