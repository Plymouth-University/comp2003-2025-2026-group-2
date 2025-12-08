<script lang="ts">
	import Icon from '$lib/assets/icon.svelte';
	import banner from '$lib/assets/banner.webp';
</script>

<svelte:head>
	<title>LogSmart - Digital Food Safety & Logging System</title>
	<meta
		name="description"
		content="LogSmart is the premier digital logging solution for the food service industry. Replace paper logs with our secure, audit-ready cloud platform."
	/>
	<link rel="canonical" href="https://logsmart.app" />
	<meta
		name="keywords"
		content="logsmart, logs, digital logs, food safety, haccp, temperature monitoring, kitchen management, audit ready, food hygiene, restaurant software, commercial kitchen, compliance, checklist app, paperless kitchen, fridge temperature log, cleaning schedule, due diligence"
	/>
</svelte:head>

<div class="bg-bg-primary flex h-full w-full flex-col">
	<section
		class="relative px-6 py-6"
		style="background-image: url('{banner}'); background-size: cover; background-position: center;"
	>
		<div class="absolute inset-0 bg-black opacity-50"></div>
		<div class="relative mx-auto max-w-7xl">
			<div class="text-center">
				<div class="mb-4 flex justify-center">
					<div class="flex h-50 w-50 justify-center rounded" aria-hidden="true">
						<Icon />
					</div>
					<p class="sr-only">LogSmart</p>
				</div>
				<h1
					class="mx-auto mb-4 max-w-5xl text-3xl leading-tight font-extrabold text-white md:text-4xl"
				>
					Digital food safety and logging that stays audit-ready
				</h1>
				<p class="mx-auto mb-6 max-w-2xl text-lg text-white">
					Introducing LogSmart, the template-based digital logging system initially designed for the
					food service industry. Replace your paper logs with an instant, secure, and eco-friendly
					solution that optimizes your daily operations.
				</p>
				<div class="flex flex-col items-center justify-center gap-3 sm:flex-row">
					<a
						href="/register-company"
						class="bg-text-primary text-bg-primary rounded px-5 py-3 text-sm font-semibold transition hover:opacity-90"
					>
						Start free trial
					</a>
					<a
						href="/contact"
						class="border-border-primary bg-bg-secondary text-text-primary rounded border px-5 py-3 text-sm font-semibold transition hover:opacity-90"
					>
						Book a demo
					</a>
				</div>
			</div>
		</div>
	</section>

	<section id="features" class="border-border-primary bg-bg-primary flex-1 border-t px-6 py-10">
		<div class="mx-auto max-w-7xl">
			<h2 class="text-text-secondary mb-3 text-sm font-semibold tracking-wide uppercase">
				Why LogSmart
			</h2>
			<h3 class="text-text-primary mb-8 text-3xl font-bold">Built for daily compliance</h3>
			<div class="grid gap-8 md:grid-cols-2 lg:grid-cols-2">
				<div class="bg-bg-secondary rounded p-6 shadow-sm">
					<h4 class="text-text-primary mb-2 text-lg font-semibold">Stay audit-ready</h4>
					<p class="text-text-secondary mb-3 text-sm">
						24/7 Access instant, tamper-proof, and audit-ready records stored securely in the cloud.
						Breeze through your next inspection with complete, organised digital logs at your
						fingertips.
					</p>
				</div>

				<div class="bg-bg-secondary rounded p-6 shadow-sm">
					<h4 class="text-text-primary mb-2 text-lg font-semibold">Optimize your workflow</h4>
					<p class="text-text-secondary mb-3 text-sm">
						Stop wasting time on manual paperwork. Our digital system frees up your team to focus on
						what matters—quality service. Streamline daily checks, reduce errors, and improve
						operational efficiency.
					</p>
				</div>

				<div class="bg-bg-secondary rounded p-6 shadow-sm">
					<h4 class="text-text-primary mb-2 text-lg font-semibold">
						Smart, customizable templates
					</h4>
					<p class="text-text-secondary mb-3 text-sm">
						Get started immediately with pre-loaded, regulatory-standard templates for all your food
						safety needs. Customize them easily to fit your specific operations, ensuring full
						compliance.
					</p>
				</div>

				<div class="bg-bg-secondary rounded p-6 shadow-sm">
					<h4 class="text-text-primary mb-2 text-lg font-semibold">Go green, go digital</h4>
					<p class="text-text-secondary mb-3 text-sm">
						Make a positive environmental impact by eliminating paper waste. Our digital solution is
						not only good for your business, but it's also good for the planet.
					</p>
				</div>
			</div>
			<div class="mt-12 grid gap-8 lg:grid-cols-2">
				<div class="bg-bg-secondary rounded p-6 shadow-sm">
					<h3 class="text-text-primary mb-3 text-2xl font-bold">How it works</h3>
					<ul class="text-text-secondary space-y-3 text-sm">
						<li>
							<span class="text-text-primary font-semibold">1. Pick a template:</span> Choose from pre-built
							food safety, cleaning, and temperature templates.
						</li>
						<li>
							<span class="text-text-primary font-semibold">2. Assign and remind:</span> Set schedules,
							owners, and reminders so checks never slip.
						</li>
						<li>
							<span class="text-text-primary font-semibold">3. Capture and store:</span> Team members
							log data on any device, stored securely in the cloud.
						</li>
						<li>
							<span class="text-text-primary font-semibold">4. Review and export:</span> Managers audit
							completion, track trends, and export reports for inspectors.
						</li>
					</ul>
				</div>
				<div class="bg-bg-secondary rounded p-6 shadow-sm">
					<h3 class="text-text-primary mb-3 text-2xl font-bold">Who it is for</h3>
					<div class="grid gap-3 md:grid-cols-2">
						<div class="border-border-primary bg-bg-primary rounded border p-4">
							<h4 class="text-text-primary text-lg font-semibold">Restaurants</h4>
							<p class="text-text-secondary text-sm">
								Keep HACCP logs current across shifts and locations.
							</p>
						</div>
						<div class="border-border-primary bg-bg-primary rounded border p-4">
							<h4 class="text-text-primary text-lg font-semibold">Hotels and catering</h4>
							<p class="text-text-secondary text-sm">
								Standardise back-of-house checks for banquets and room service.
							</p>
						</div>
						<div class="border-border-primary bg-bg-primary rounded border p-4">
							<h4 class="text-text-primary text-lg font-semibold">Education</h4>
							<p class="text-text-secondary text-sm">
								Run consistent compliance in cafeterias without paper binders.
							</p>
						</div>
						<div class="border-border-primary bg-bg-primary rounded border p-4">
							<h4 class="text-text-primary text-lg font-semibold">Franchises</h4>
							<p class="text-text-secondary text-sm">
								Roll out templates network-wide and monitor completion in real time.
							</p>
						</div>
					</div>
				</div>
			</div>
			<div class="mt-12 grid gap-6 md:grid-cols-2">
				<div class="bg-bg-secondary rounded p-6 shadow-sm">
					<h3 class="text-text-primary mb-3 text-2xl font-bold">What you get</h3>
					<ul class="text-text-secondary space-y-2 text-sm">
						<li>Real-time completion dashboards and alerts.</li>
						<li>Offline-friendly logging for low-connectivity kitchens.</li>
						<li>Role-based access so the right people see the right logs.</li>
						<li>PDF and CSV exports for auditors and management.</li>
						<li>Secure storage with backups and permissions.</li>
					</ul>
				</div>
				<div class="bg-bg-secondary rounded p-6 shadow-sm">
					<h3 class="text-text-primary mb-3 text-2xl font-bold">Frequently asked questions</h3>
					<dl class="text-text-secondary space-y-4 text-sm">
						<div>
							<dt class="text-text-primary font-semibold">How fast can we go live?</dt>
							<dd>Most teams launch in under a week using the pre-built templates.</dd>
						</div>
						<div>
							<dt class="text-text-primary font-semibold">Do you support multiple sites?</dt>
							<dd>Yes. Manage locations centrally and view completion by site.</dd>
						</div>
						<div>
							<dt class="text-text-primary font-semibold">What about inspections?</dt>
							<dd>Share digital reports or exports with inspectors to speed up audits.</dd>
						</div>
					</dl>
				</div>
			</div>
		</div>
	</section>

	<section class="border-border-primary bg-bg-primary border-t px-6 py-12">
		<div class="mx-auto max-w-7xl text-center">
			<p class="text-text-secondary">© 2025 LogSmart. All rights reserved.</p>
		</div>
	</section>
</div>
