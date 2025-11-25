<script lang="ts">
	import { goto, invalidateAll } from '$app/navigation';

	function validatePassword(pwd: string): { valid: boolean; errors: string[] } {
		const errors: string[] = [];
		if (pwd.length < 8) errors.push('at least 8 characters');
		if (!/[A-Z]/.test(pwd)) errors.push('an uppercase letter');
		if (!/[a-z]/.test(pwd)) errors.push('a lowercase letter');
		if (!/\d/.test(pwd)) errors.push('a digit');
		if (!/[!@#$%^&*()_+\-=\[\]{};':"\\|,.<>/?]/.test(pwd)) errors.push('a special character');
		return { valid: errors.length === 0, errors };
	}

	let step = $state(1);
	let companyName = $state('');
	let companyAddress = $state('');
	let firstName = $state('');
	let lastName = $state('');
	let email = $state('');
	let password = $state('');
	let confirmPassword = $state('');
	let loading = $state(false);
	let error = $state('');
	let passwordErrors = $state<string[]>([]);
	let showPassword = $state(false);
	let showConfirmPassword = $state(false);
	let touched = $state({
		companyName: false,
		companyAddress: false,
		firstName: false,
		lastName: false,
		email: false,
		password: false,
		confirmPassword: false
	});

	const companyNameValid = $derived(companyName.trim().length > 0);
	const companyAddressValid = $derived(companyAddress.trim().length > 0);
	const firstNameValid = $derived(firstName.trim().length > 0);
	const lastNameValid = $derived(lastName.trim().length > 0);
	const emailValid = $derived(/^\S+@\S+\.\S+$/.test(email));
	const passwordValid = $derived(validatePassword(password).valid);
	const passwordsMatch = $derived(password === confirmPassword && confirmPassword.length > 0);

	const step1Valid = $derived(companyNameValid && companyAddressValid);
	const step2Valid = $derived(
		firstNameValid && lastNameValid && emailValid && passwordValid && passwordsMatch
	);

	$effect(() => {
		if (password) {
			const validation = validatePassword(password);
			passwordErrors = validation.errors;
		} else {
			passwordErrors = [];
		}
	});

	function nextStep(e: Event) {
		e.preventDefault();
		touched.companyName = true;
		touched.companyAddress = true;
		if (step1Valid) {
			step = 2;
		}
	}

	function prevStep() {
		step = 1;
		error = '';
	}

	async function submit(e: Event) {
		e.preventDefault();
		error = '';
		touched = {
			companyName: true,
			companyAddress: true,
			firstName: true,
			lastName: true,
			email: true,
			password: true,
			confirmPassword: true
		};

		if (!step2Valid) return;

		loading = true;
		try {
			const res = await fetch('/api/auth/register', {
				method: 'POST',
				headers: { 'Content-Type': 'application/json' },
				body: JSON.stringify({
					company_name: companyName,
					company_address: companyAddress,
					first_name: firstName,
					last_name: lastName,
					email,
					password
				})
			});

			if (!res.ok) {
				const data = await res.json().catch(() => ({}));
				error = data?.message || 'Registration failed';
			} else {
				await invalidateAll();
				await goto('/dashboard');
			}
		} catch {
			error = 'Network error';
		} finally {
			loading = false;
		}
	}
</script>

<svelte:head>
	<title>Register Company - LogSmart</title>
</svelte:head>

<div class="register-wrapper">
	<div class="form-container">
		<div class="progress-indicator">
			<div class="step" class:active={step === 1} class:completed={step > 1}>
				<div class="step-number">1</div>
				<div class="step-label">Company Details</div>
			</div>
			<div class="progress-line" class:completed={step > 1}></div>
			<div class="step" class:active={step === 2}>
				<div class="step-number">2</div>
				<div class="step-label">Your Details</div>
			</div>
		</div>

		{#if step === 1}
			<form onsubmit={nextStep} class="form">
				<h1>Company Information</h1>
				<p class="subtitle">Tell us about your company</p>

				{#if error}
					<div class="error" role="alert">{error}</div>
				{/if}

				<label class="field">
					<span class="label-text">Company Name</span>
					<input
						type="text"
						bind:value={companyName}
						onblur={() => (touched.companyName = true)}
						aria-invalid={!companyNameValid}
						placeholder="LogSmart Ltd"
						required
					/>
					{#if touched.companyName && !companyNameValid}
						<div class="field-error">Company name is required.</div>
					{/if}
				</label>

				<label class="field">
					<span class="label-text">Company Address</span>
					<textarea
						bind:value={companyAddress}
						onblur={() => (touched.companyAddress = true)}
						aria-invalid={!companyAddressValid}
						placeholder="Plymouth, UK"
						rows="3"
						required
					></textarea>
					{#if touched.companyAddress && !companyAddressValid}
						<div class="field-error">Company address is required.</div>
					{/if}
				</label>

				<button type="submit" class="btn btn-primary" disabled={!step1Valid}> Next Step </button>
			</form>
		{:else}
			<form onsubmit={submit} class="form">
				<h1>Your Details</h1>
				<p class="subtitle">Create your admin account</p>

				{#if error}
					<div class="error" role="alert">{error}</div>
				{/if}

				<div class="field-row">
					<label class="field">
						<span class="label-text">First Name</span>
						<input
							type="text"
							bind:value={firstName}
							onblur={() => (touched.firstName = true)}
							aria-invalid={!firstNameValid}
							placeholder="John"
							required
						/>
						{#if touched.firstName && !firstNameValid}
							<div class="field-error">First name is required.</div>
						{/if}
					</label>

					<label class="field">
						<span class="label-text">Last Name</span>
						<input
							type="text"
							bind:value={lastName}
							onblur={() => (touched.lastName = true)}
							aria-invalid={!lastNameValid}
							placeholder="Doe"
							required
						/>
						{#if touched.lastName && !lastNameValid}
							<div class="field-error">Last name is required.</div>
						{/if}
					</label>
				</div>

				<label class="field">
					<span class="label-text">Email</span>
					<input
						type="email"
						bind:value={email}
						onblur={() => (touched.email = true)}
						aria-invalid={!emailValid}
						placeholder="john@company.com"
						required
					/>
					{#if touched.email && !emailValid}
						<div class="field-error">Enter a valid email address.</div>
					{/if}
				</label>

				<label class="field">
					<span class="label-text">Password</span>
					<div class="password-input-wrapper">
						<input
							type={showPassword ? 'text' : 'password'}
							bind:value={password}
							onblur={() => (touched.password = true)}
							aria-invalid={!passwordValid}
							placeholder="Min 8 chars, uppercase, lowercase, digit, special char"
							class="password-input"
							required
						/>
						<button
							type="button"
							class="password-toggle"
							onclick={() => (showPassword = !showPassword)}
							aria-label={showPassword ? 'Hide password' : 'Show password'}
						>
							{#if showPassword}
								<svg width="20" height="20" fill="none" stroke="currentColor" viewBox="0 0 24 24">
									<path
										stroke-linecap="round"
										stroke-linejoin="round"
										stroke-width="2"
										d="M13.875 18.825A10.05 10.05 0 0112 19c-4.478 0-8.268-2.943-9.543-7a9.97 9.97 0 011.563-3.029m5.858.908a3 3 0 114.243 4.243M9.878 9.878l4.242 4.242M9.88 9.88l-3.29-3.29m7.532 7.532l3.29 3.29M3 3l3.59 3.59m0 0A9.953 9.953 0 0112 5c4.478 0 8.268 2.943 9.543 7a10.025 10.025 0 01-4.132 5.411m0 0L21 21"
									/>
								</svg>
							{:else}
								<svg width="20" height="20" fill="none" stroke="currentColor" viewBox="0 0 24 24">
									<path
										stroke-linecap="round"
										stroke-linejoin="round"
										stroke-width="2"
										d="M15 12a3 3 0 11-6 0 3 3 0 016 0z"
									/>
									<path
										stroke-linecap="round"
										stroke-linejoin="round"
										stroke-width="2"
										d="M2.458 12C3.732 7.943 7.523 5 12 5c4.478 0 8.268 2.943 9.542 7-1.274 4.057-5.064 7-9.542 7-4.477 0-8.268-2.943-9.542-7z"
									/>
								</svg>
							{/if}
						</button>
					</div>
					{#if password}
						<div class="mt-2 space-y-1 text-xs">
							<div class={passwordErrors.length === 0 ? 'text-green-600' : 'text-gray-500'}>
								✓ Password requirements
							</div>
							<div class={!/[A-Z]/.test(password) ? 'text-red-600' : 'text-green-600'}>
								{!/[A-Z]/.test(password) ? '✗' : '✓'} Uppercase letter (A-Z)
							</div>
							<div class={!/[a-z]/.test(password) ? 'text-red-600' : 'text-green-600'}>
								{!/[a-z]/.test(password) ? '✗' : '✓'} Lowercase letter (a-z)
							</div>
							<div class={!/\d/.test(password) ? 'text-red-600' : 'text-green-600'}>
								{!/\d/.test(password) ? '✗' : '✓'} Digit (0-9)
							</div>
							<div
								class={!/[!@#$%^&*()_+\-=\[\]{};':"\\|,.<>/?]/.test(password)
									? 'text-red-600'
									: 'text-green-600'}
							>
								{!/[!@#$%^&*()_+\-=\[\]{};':"\\|,.<>/?]/.test(password) ? '✗' : '✓'} Special character
								(!@#$%^&* etc)
							</div>
							<div class={password.length < 8 ? 'text-red-600' : 'text-green-600'}>
								{password.length < 8 ? '✗' : '✓'} At least 8 characters
							</div>
						</div>
					{/if}
					{#if touched.password && !passwordValid}
						<div class="field-error">Password must meet all requirements.</div>
					{/if}
				</label>

				<label class="field">
					<span class="label-text">Confirm Password</span>
					<div class="password-input-wrapper">
						<input
							type={showConfirmPassword ? 'text' : 'password'}
							bind:value={confirmPassword}
							onblur={() => (touched.confirmPassword = true)}
							aria-invalid={!passwordsMatch}
							placeholder="Re-enter password"
							class="password-input"
							required
						/>
						<button
							type="button"
							class="password-toggle"
							onclick={() => (showConfirmPassword = !showConfirmPassword)}
							aria-label={showConfirmPassword ? 'Hide password' : 'Show password'}
						>
							{#if showConfirmPassword}
								<svg width="20" height="20" fill="none" stroke="currentColor" viewBox="0 0 24 24">
									<path
										stroke-linecap="round"
										stroke-linejoin="round"
										stroke-width="2"
										d="M13.875 18.825A10.05 10.05 0 0112 19c-4.478 0-8.268-2.943-9.543-7a9.97 9.97 0 011.563-3.029m5.858.908a3 3 0 114.243 4.243M9.878 9.878l4.242 4.242M9.88 9.88l-3.29-3.29m7.532 7.532l3.29 3.29M3 3l3.59 3.59m0 0A9.953 9.953 0 0112 5c4.478 0 8.268 2.943 9.543 7a10.025 10.025 0 01-4.132 5.411m0 0L21 21"
									/>
								</svg>
							{:else}
								<svg width="20" height="20" fill="none" stroke="currentColor" viewBox="0 0 24 24">
									<path
										stroke-linecap="round"
										stroke-linejoin="round"
										stroke-width="2"
										d="M15 12a3 3 0 11-6 0 3 3 0 016 0z"
									/>
									<path
										stroke-linecap="round"
										stroke-linejoin="round"
										stroke-width="2"
										d="M2.458 12C3.732 7.943 7.523 5 12 5c4.478 0 8.268 2.943 9.542 7-1.274 4.057-5.064 7-9.542 7-4.477 0-8.268-2.943-9.542-7z"
									/>
								</svg>
							{/if}
						</button>
					</div>
					{#if touched.confirmPassword && !passwordsMatch}
						<div class="field-error">Passwords do not match.</div>
					{/if}
				</label>

				<div class="button-group">
					<button type="button" class="btn btn-secondary" onclick={prevStep}> Back </button>
					<button type="submit" class="btn btn-primary" disabled={!step2Valid || loading}>
						{#if loading}
							Creating Account...
						{:else}
							Create Account
						{/if}
					</button>
				</div>
			</form>
		{/if}

		<p class="footer-text">
			Already have an account? <a href="/login">Sign in</a>
		</p>
	</div>
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
		background: #f5f7fb;
	}

	.register-wrapper {
		width: 100%;
		height: 100%;
		display: flex;
		align-items: center;
		justify-content: center;
		padding: 2rem;
		box-sizing: border-box;
	}

	.form-container {
		width: 100%;
		max-width: 600px;
	}

	.progress-indicator {
		display: flex;
		align-items: center;
		justify-content: center;
		margin-bottom: 2rem;
		gap: 1rem;
	}

	.step {
		display: flex;
		flex-direction: column;
		align-items: center;
		gap: 0.5rem;
	}

	.step-number {
		width: 40px;
		height: 40px;
		border-radius: 50%;
		background: #e5e7eb;
		color: #6b7280;
		display: flex;
		align-items: center;
		justify-content: center;
		font-weight: 600;
		transition: all 0.3s;
	}

	.step.active .step-number {
		background: #2f6fff;
		color: white;
	}

	.step.completed .step-number {
		background: #10b981;
		color: white;
	}

	.step-label {
		font-size: 0.875rem;
		color: #6b7280;
	}

	.step.active .step-label {
		color: #111827;
		font-weight: 600;
	}

	.progress-line {
		width: 80px;
		height: 2px;
		background: #e5e7eb;
		transition: all 0.3s;
	}

	.progress-line.completed {
		background: #10b981;
	}

	.form {
		width: 100%;
		background: white;
		padding: 2rem;
		border-radius: 8px;
		box-shadow: 0 6px 20px rgba(20, 20, 50, 0.08);
	}

	h1 {
		margin: 0 0 0.5rem 0;
		font-size: 1.5rem;
	}

	.subtitle {
		color: #6b7280;
		margin: 0 0 1.5rem 0;
	}

	.field {
		display: flex;
		flex-direction: column;
		margin-bottom: 1rem;
	}

	.field-row {
		display: grid;
		grid-template-columns: 1fr 1fr;
		gap: 1rem;
	}

	.label-text {
		font-size: 0.9rem;
		font-weight: 500;
		margin-bottom: 0.25rem;
	}

	input,
	textarea {
		padding: 0.5rem 0.75rem;
		border: 1px solid #d1d7e0;
		border-radius: 6px;
		font-size: 1rem;
		outline: none;
		font-family: inherit;
		width: 100%;
		box-sizing: border-box;
	}

	input:focus,
	textarea:focus {
		border-color: #6b8cff;
		box-shadow: 0 0 0 3px rgba(107, 140, 255, 0.12);
	}

	textarea {
		resize: vertical;
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
		padding: 0.75rem 1.5rem;
		border: none;
		border-radius: 6px;
		font-size: 1rem;
		cursor: pointer;
		font-weight: 500;
		transition: all 0.2s;
	}

	.btn-primary {
		width: 100%;
		background: #2f6fff;
		color: white;
	}

	.btn-primary:hover:not([disabled]) {
		background: #1e5cff;
	}

	.btn-secondary {
		background: #f3f4f6;
		color: #374151;
	}

	.btn-secondary:hover {
		background: #e5e7eb;
	}

	.btn[disabled] {
		opacity: 0.6;
		cursor: not-allowed;
	}

	.button-group {
		display: flex;
		gap: 1rem;
		margin-top: 1.5rem;
	}

	.button-group .btn-primary {
		flex: 1;
	}

	.footer-text {
		text-align: center;
		margin-top: 1.5rem;
		color: #6b7280;
	}

	.footer-text a {
		color: #2f6fff;
		text-decoration: none;
	}

	.footer-text a:hover {
		text-decoration: underline;
	}

	.password-input-wrapper {
		position: relative;
		display: block;
	}

	.password-input {
		padding-right: 2.5rem !important;
	}

	.password-toggle {
		position: absolute;
		right: 0.5rem;
		top: 50%;
		transform: translateY(-50%);
		background: none;
		border: none;
		cursor: pointer;
		color: #6b7280;
		padding: 0.25rem;
		display: flex;
		align-items: center;
		justify-content: center;
	}

	.password-toggle:hover {
		color: #374151;
	}
</style>
