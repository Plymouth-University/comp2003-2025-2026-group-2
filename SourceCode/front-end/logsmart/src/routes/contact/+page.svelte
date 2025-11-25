<script lang="ts">
	let email = $state('');
	let name = $state('');
	let message = $state('');
	let submitted = $state(false);

	function handleSubmit(e: Event) {
		e.preventDefault();
		window.location.href = `mailto:team@logsmart.app?subject=Contact from ${encodeURIComponent(name)}&body=${encodeURIComponent(message)}`;
		submitted = true;
		setTimeout(() => {
			submitted = false;
			name = '';
			email = '';
			message = '';
		}, 3000);
	}
</script>

<svelte:head>
	<title>Contact Us - LogSmart</title>
</svelte:head>

<div class="flex h-full w-full flex-col bg-gray-50">
	<div class="mx-auto max-w-4xl flex-1 px-6 py-16">
		<div class="mb-8 text-center">
			<h1 class="mb-4 text-4xl font-bold text-gray-900">Get in Touch</h1>
			<p class="text-lg text-gray-600">
				Have questions about LogSmart? We'd love to hear from you.
			</p>
		</div>

		<div class="rounded-lg bg-white p-8 shadow-sm">
			<div class="mb-8 rounded-lg bg-blue-50 p-6 text-center">
				<svg
					class="mx-auto mb-4 h-12 w-12 text-blue-600"
					fill="none"
					viewBox="0 0 24 24"
					stroke="currentColor"
				>
					<path
						stroke-linecap="round"
						stroke-linejoin="round"
						stroke-width="2"
						d="M3 8l7.89 5.26a2 2 0 002.22 0L21 8M5 19h14a2 2 0 002-2V7a2 2 0 00-2-2H5a2 2 0 00-2 2v10a2 2 0 002 2z"
					/>
				</svg>
				<h2 class="mb-2 text-2xl font-bold text-gray-900">Email Us Directly</h2>
				<a
					href="mailto:team@logsmart.app"
					class="text-xl font-semibold text-blue-600 hover:text-blue-700"
				>
					team@logsmart.app
				</a>
				<p class="mt-3 text-gray-600">
					Click the email above or use the form below to reach our team
				</p>
			</div>

			{#if submitted}
				<div class="mb-6 rounded-lg bg-green-50 p-4 text-center">
					<p class="font-semibold text-green-800">Your email client should open shortly!</p>
				</div>
			{/if}

			<form onsubmit={handleSubmit} class="space-y-6">
				<div>
					<label for="name" class="mb-2 block text-sm font-semibold text-gray-700">
						Your Name
					</label>
					<input
						type="text"
						id="name"
						bind:value={name}
						required
						class="w-full rounded border border-gray-300 px-4 py-2 focus:border-blue-500 focus:ring-2 focus:ring-blue-500 focus:outline-none"
						placeholder="John Doe"
					/>
				</div>

				<div>
					<label for="email" class="mb-2 block text-sm font-semibold text-gray-700">
						Your Email
					</label>
					<input
						type="email"
						id="email"
						bind:value={email}
						required
						class="w-full rounded border border-gray-300 px-4 py-2 focus:border-blue-500 focus:ring-2 focus:ring-blue-500 focus:outline-none"
						placeholder="you@company.com"
					/>
				</div>

				<div>
					<label for="message" class="mb-2 block text-sm font-semibold text-gray-700">
						Message
					</label>
					<textarea
						id="message"
						bind:value={message}
						required
						rows="6"
						class="w-full rounded border border-gray-300 px-4 py-2 focus:border-blue-500 focus:ring-2 focus:ring-blue-500 focus:outline-none"
						placeholder="Tell us about your needs..."
					></textarea>
				</div>

				<button
					type="submit"
					class="w-full rounded bg-blue-600 px-6 py-3 font-semibold text-white hover:bg-blue-700 focus:ring-2 focus:ring-blue-500 focus:ring-offset-2 focus:outline-none"
				>
					Send Message
				</button>
			</form>

			<div class="mt-8 border-t border-gray-200 pt-8">
				<h3 class="mb-4 text-lg font-semibold text-gray-900">Other Ways to Reach Us</h3>
				<div class="space-y-3 text-gray-600">
					<p>
						<span class="font-semibold">Support:</span> For technical assistance or account issues, email
						us directly at team@logsmart.app
					</p>
					<p>
						<span class="font-semibold">Sales:</span> Interested in LogSmart for your business? Contact
						our team to learn more about pricing and features
					</p>
					<p>
						<span class="font-semibold">Partnerships:</span> Exploring collaboration opportunities? We'd
						love to discuss how we can work together
					</p>
				</div>
			</div>
		</div>
	</div>

	<footer class="border-t border-gray-300 bg-white px-6 py-12">
		<div class="mx-auto max-w-7xl text-center">
			<p class="text-gray-600">© 2025 LogSmart. All rights reserved.</p>
		</div>
	</footer>
</div>
