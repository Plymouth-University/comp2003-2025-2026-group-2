<script lang="ts">
	import Icon from '$lib/assets/icon.svelte';
	import banner from '$lib/assets/banner.webp';
</script>

<div class="flex h-full w-full flex-col bg-bg-primary">
	<section
		class="relative px-6 py-6"
		style="background-image: url('{banner}'); background-size: cover; background-position: center;"
	>
		<!-- Dark overlay for contrast -->
		<div class="absolute inset-0 bg-black opacity-50"></div>

		<!-- Content -->
		<div class="relative mx-auto max-w-7xl">
			<div class="text-center">
				<div class="mb-4 flex justify-center">
					<div class="flex h-50 w-50 items-center justify-center rounded">
						<Icon />
					</div>
				</div>
				<p class="mx-auto mb-4 max-w-2xl text-lg text-white">
					Introducing LogSmart, the template-based digital logging system initially designed for the
					food service industry. Replace your paper logs with an instant, secure, and eco-friendly
					solution that optimizes your daily operations.
				</p>
			</div>
		</div>
	</section>

	<section id="features" class="flex-1 border-t border-border-primary bg-bg-primary px-6 py-10">
		<div class="mx-auto max-w-7xl">
			<h3 class="mb-8 text-3xl font-bold text-text-primary">Features</h3>
			<div class="grid gap-8 md:grid-cols-2 lg:grid-cols-2">
				<div class="rounded bg-bg-secondary p-6 shadow-sm">
					<h4 class="mb-2 text-lg font-semibold text-text-primary">Feature 1: Stay Audit-Ready</h4>
					<p class="mb-3 text-sm text-text-secondary">
						24/7 Access instant, tamper-proof, and audit-ready records stored securely in the cloud.
						Breeze through your next inspection with complete, organised digital logs at your
						fingertips.
					</p>
				</div>

				<div class="rounded bg-bg-secondary p-6 shadow-sm">
					<h4 class="mb-2 text-lg font-semibold text-text-primary">
						Feature 2: Optimize Your Workflow
					</h4>
					<p class="mb-3 text-sm text-text-secondary">
						Stop wasting time on manual paperwork. Our digital system frees up your team to focus on
						what matters—quality service. Streamline daily checks, reduce errors, and improve
						operational efficiency.
					</p>
				</div>

				<div class="rounded bg-bg-secondary p-6 shadow-sm">
					<h4 class="mb-2 text-lg font-semibold text-text-primary">
						Feature 3: Smart, Customizable Templates
					</h4>
					<p class="mb-3 text-sm text-text-secondary">
						Get started immediately with pre-loaded, regulatory-standard templates for all your food
						safety needs. Customize them easily to fit your specific operations, ensuring full
						compliance.
					</p>
				</div>

				<div class="rounded bg-bg-secondary p-6 shadow-sm">
					<h4 class="mb-2 text-lg font-semibold text-text-primary">
						Feature 4: Go Green, Go Digital
					</h4>
					<p class="mb-3 text-sm text-text-secondary">
						Make a positive environmental impact by eliminating paper waste. Our digital solution is
						not only good for your business, but it's also good for the planet.
					</p>
				</div>
			</div>
		</div>
	</section>

	<section class="border-t border-border-primary bg-bg-primary px-6 py-12">
		<div class="mx-auto max-w-7xl text-center">
			<p class="text-text-secondary">© 2025 LogSmart. All rights reserved.</p>
		</div>
	</section>
</div>
