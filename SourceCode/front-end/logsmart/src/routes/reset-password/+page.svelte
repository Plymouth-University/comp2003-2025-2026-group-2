<script lang="ts">
	function validatePassword(pwd: string): { valid: boolean; errors: string[] } {
		const errors: string[] = [];
		if (pwd.length < 8) errors.push('at least 8 characters');
		if (!/[A-Z]/.test(pwd)) errors.push('an uppercase letter');
		if (!/[a-z]/.test(pwd)) errors.push('a lowercase letter');
		if (!/\d/.test(pwd)) errors.push('a digit');
		if (!/[!@#$%^&*()_+\-=\[\]{};':"\\|,.<>/?]/.test(pwd))
			errors.push('a special character');
		return { valid: errors.length === 0, errors };
	}

	let newPassword = $state('');
	let confirmPassword = $state('');
	let email = $state('');
	let status = $state<'idle' | 'submitting' | 'success' | 'error'>('idle');
	let message = $state('');
	let token = $state('');
	let hasToken = $state(false);
	let passwordErrors = $state<string[]>([]);

	$effect.pre(() => {
		const params = new URLSearchParams(window.location.search);
		const t = params.get('token');
		if (t) {
			token = t;
			hasToken = true;
		} else {
			hasToken = false;
		}
	});

	$effect(() => {
		if (newPassword) {
			const validation = validatePassword(newPassword);
			passwordErrors = validation.errors;
		} else {
			passwordErrors = [];
		}
	});

	async function handleResetSubmit(event: SubmitEvent) {
		event.preventDefault();
		if (!email) {
			status = 'error';
			message = 'Please enter your email address.';
			return;
		}

		status = 'submitting';
		message = '';

		const response = await fetch('/api/auth/password/request-reset', {
			method: 'POST',
			headers: { 'Content-Type': 'application/json' },
			body: JSON.stringify({ email })
		});

		if (response.ok) {
			status = 'success';
			message = 'Reset link sent to your email. Please check your inbox.';
			email = '';
		} else {
			const body = await response.json();
			status = 'error';
			message = body.error ?? 'Unable to send reset link.';
		}
	}

	async function handlePasswordSubmit(event: SubmitEvent) {
		event.preventDefault();
		const pwdValidation = validatePassword(newPassword);
		if (!pwdValidation.valid) {
			status = 'error';
			message = `Password must include ${pwdValidation.errors.join(', ')}.`;
			return;
		}
		if (newPassword !== confirmPassword) {
			status = 'error';
			message = 'Passwords do not match.';
			return;
		}
		if (!token) {
			status = 'error';
			message = 'Invalid reset token.';
			return;
		}

		status = 'submitting';
		message = '';

		const response = await fetch('/api/auth/password/reset', {
			method: 'POST',
			headers: { 'Content-Type': 'application/json' },
			body: JSON.stringify({ token, new_password: newPassword })
		});

		if (response.ok) {
			const body = await response.json();
			status = 'success';
			message = body.message;
		} else {
			const body = await response.json();
			status = 'error';
			message = body.error ?? 'Unable to reset your password.';
		}
	}
</script>

<svelte:head>
	<title>Reset Password | LogSmart</title>
</svelte:head>

<div class="flex min-h-full w-full flex-1 items-center justify-center bg-gray-50">
	<section class="w-full max-w-lg flex-1 space-y-6 rounded-lg bg-white p-8 shadow-lg">
		<div>
			<h1 class="text-2xl font-semibold text-gray-800">Reset your password</h1>
			<p class="mt-1 text-sm text-gray-500">
				{#if hasToken}
					Enter a new secure password for your account.
				{:else}
					Enter your email to receive a password reset link.
				{/if}
			</p>
		</div>

		{#if status === 'error'}
			<div class="rounded border border-red-300 bg-red-50 p-3 text-sm text-red-700">
				{message}
			</div>
		{/if}

		{#if status === 'success'}
			<div class="rounded border border-green-300 bg-green-50 p-3 text-sm text-green-700">
				{message}
			</div>
		{/if}

		{#if hasToken}
			<form class="space-y-4" onsubmit={handlePasswordSubmit}>
				<div>
					<label for="newPassword" class="text-sm font-medium text-gray-700">New password</label>
					<input
						id="newPassword"
						type="password"
						bind:value={newPassword}
						class="mt-1 w-full rounded border px-3 py-2"
						required
					/>
					{#if newPassword}
						<div class="mt-2 space-y-1 text-xs">
							<div class={passwordErrors.length === 0 ? 'text-green-600' : 'text-gray-500'}>
								✓ Password requirements
							</div>
							<div class={!/[A-Z]/.test(newPassword) ? 'text-red-600' : 'text-green-600'}>
								{!/[A-Z]/.test(newPassword) ? '✗' : '✓'} Uppercase letter (A-Z)
							</div>
							<div class={!/[a-z]/.test(newPassword) ? 'text-red-600' : 'text-green-600'}>
								{!/[a-z]/.test(newPassword) ? '✗' : '✓'} Lowercase letter (a-z)
							</div>
							<div class={!/\d/.test(newPassword) ? 'text-red-600' : 'text-green-600'}>
								{!/\d/.test(newPassword) ? '✗' : '✓'} Digit (0-9)
							</div>
							<div
								class={!/[!@#$%^&*()_+\-=\[\]{};':"\\|,.<>/?]/.test(newPassword)
									? 'text-red-600'
									: 'text-green-600'}
							>
								{!/[!@#$%^&*()_+\-=\[\]{};':"\\|,.<>/?]/.test(newPassword)
									? '✗'
									: '✓'} Special character (!@#$%^&* etc)
							</div>
							<div class={newPassword.length < 8 ? 'text-red-600' : 'text-green-600'}>
								{newPassword.length < 8 ? '✗' : '✓'} At least 8 characters
							</div>
						</div>
					{/if}
				</div>
				<div>
					<label for="confirmPassword" class="text-sm font-medium text-gray-700"
						>Confirm password</label
					>
					<input
						id="confirmPassword"
						type="password"
						bind:value={confirmPassword}
						class="mt-1 w-full rounded border px-3 py-2"
						required
					/>
				</div>
				<button
					class="w-full rounded bg-indigo-600 px-4 py-2 font-semibold text-white transition hover:bg-indigo-500 disabled:opacity-50"
					type="submit"
					disabled={status === 'submitting' || status === 'success' || passwordErrors.length > 0 || !newPassword || !confirmPassword || newPassword !== confirmPassword}
				>
					{#if status === 'submitting'}
						Updating...
					{:else}
						Set new password
					{/if}
				</button>
			</form>
		{:else}
			<form class="space-y-4" onsubmit={handleResetSubmit}>
				<div>
					<label for="email" class="text-sm font-medium text-gray-700">Email address</label>
					<input
						id="email"
						type="email"
						bind:value={email}
						class="mt-1 w-full rounded border px-3 py-2"
						required
					/>
				</div>
				<button
					class="w-full rounded bg-indigo-600 px-4 py-2 font-semibold text-white transition hover:bg-indigo-500"
					type="submit"
					disabled={status === 'submitting' || status === 'success'}
				>
					{#if status === 'submitting'}
						Sending...
					{:else}
						Send reset link
					{/if}
				</button>
			</form>
		{/if}
	</section>
</div>
