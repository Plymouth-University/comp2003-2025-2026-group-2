<script lang="ts">
  let newPassword = $state('');
  let confirmPassword = $state('');
  let status = $state<'idle' | 'submitting' | 'success' | 'error'>('idle');
  let message = $state('');
  let token = $state('');

  $effect.pre(() => {
    const params = new URLSearchParams(window.location.search);
    const t = params.get('token');
    if (t) {
      token = t;
    } else {
      status = 'error';
      message = 'Reset token missing from the link.';
    }
  });

  async function handleSubmit(event: SubmitEvent) {
    event.preventDefault();
    if (newPassword.length < 8) {
      status = 'error';
      message = 'Choose a password with at least 8 characters.';
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

<main class="min-h-screen bg-gray-50 flex items-center justify-center">
  <section class="w-full max-w-lg bg-white shadow-lg rounded-lg p-8 space-y-6">
    <div>
      <h1 class="text-2xl font-semibold text-gray-800">Reset your password</h1>
      <p class="text-sm text-gray-500 mt-1">Enter a new secure password for your account.</p>
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

    <form class="space-y-4" onsubmit={handleSubmit}>
      <div>
        <label for="newPassword" class="text-sm font-medium text-gray-700">New password</label>
        <input
          id="newPassword"
          type="password"
          bind:value={newPassword}
          class="mt-1 w-full rounded border px-3 py-2" 
          minlength="8"
          required
        />
      </div>
      <div>
        <label for="confirmPassword" class="text-sm font-medium text-gray-700">Confirm password</label>
        <input
          id="confirmPassword"
          type="password"
          bind:value={confirmPassword}
          class="mt-1 w-full rounded border px-3 py-2"
          required
        />
      </div>
      <button
        class="w-full rounded bg-indigo-600 text-white font-semibold px-4 py-2 hover:bg-indigo-500 transition"
        type="submit"
        disabled={status === 'submitting' || status === 'success'}
      >
        {#if status === 'submitting'}
          Updating...
        {:else}
          Set new password
        {/if}
      </button>
    </form>
  </section>
</main>
