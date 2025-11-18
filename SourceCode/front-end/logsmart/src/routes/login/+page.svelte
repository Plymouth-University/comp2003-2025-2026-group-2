<script lang="ts">
import { goto } from '$app/navigation';
let email = '';
let password = '';
let loading = false;
let error = '';
let touched = { email: false, password: false };
$: emailValid = /^\S+@\S+\.\S+$/.test(email);
$: passwordValid = password.length >= 6;
$: formValid = emailValid && passwordValid;
async function submit() {
    error = '';
    touched = { email: true, password: true };
    if (!formValid) return;
    loading = true;
    try {
        const res = await fetch('/api/auth/login', {
            method: 'POST',
            headers: { 'Content-Type': 'application/json' },
            body: JSON.stringify({ email, password })
        });
        if (!res.ok) {
            const data = await res.json().catch(() => ({}));
            error = data?.error || data?.message || 'Login failed';
        } else {
            await goto('/dashboard');
        }
    } catch (err) {
        console.error('Login error:', err);
        error = 'Network error';
    } finally {
        loading = false;
    }
}
</script>

<main class="bg-gray-50 grid place-items-center p-6">
    <form on:submit|preventDefault={submit} class="form">
        <h1>Log in</h1>
        {#if error}
            <div class="error" role="alert">{error}</div>
        {/if}
        <label class="field">
            <span class="label-text">Email</span>
            <input
                type="email"
                bind:value={email}
                on:blur={() => (touched.email = true)}
                aria-invalid={!emailValid}
                aria-describedby="email-help"
                required
            />
            {#if touched.email && !emailValid}
                <div id="email-help" class="field-error">Enter a valid email address.</div>
            {/if}
        </label>

        <label class="field">
            <span class="label-text">Password</span>
            <input
                type="password"
                bind:value={password}
                on:blur={() => (touched.password = true)}
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
    </form>
</main>

<style>
:global(body) {
    font-family: system-ui, -apple-system, "Segoe UI", Roboto, "Helvetica Neue", Arial;
    margin: 0;
    padding: 0;
    background: #f5f7fb;
}
.form {
    width: 100%;
    max-width: 400px;
    background: white;
    padding: 2rem;
    border-radius: 8px;
    box-shadow: 0 6px 20px rgba(20,20,50,0.08);
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
    box-shadow: 0 0 0 3px rgba(107,140,255,0.12);
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