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

<div class="flex h-full w-full flex-col bg-bg-primary">
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
						class="rounded bg-text-primary px-5 py-3 text-sm font-semibold text-bg-primary transition hover:opacity-90"
					>
						Start free trial
					</a>
					<a
						href="/contact"
						class="rounded border border-border-primary bg-bg-secondary px-5 py-3 text-sm font-semibold text-text-primary transition hover:opacity-90"
					>
						Book a demo
					</a>
				</div>
			</div>
		</div>
	</section>

	<section id="features" class="flex-1 border-t border-border-primary bg-bg-primary px-6 py-10">
		<div class="mx-auto max-w-7xl">
			<h2 class="mb-3 text-sm font-semibold tracking-wide text-text-secondary uppercase">
				Why LogSmart
			</h2>
			<h3 class="mb-8 text-3xl font-bold text-text-primary">Built for daily compliance</h3>
			<div class="grid gap-8 md:grid-cols-2 lg:grid-cols-2">
				<div class="rounded bg-bg-secondary p-6 shadow-sm">
					<h4 class="mb-2 text-lg font-semibold text-text-primary">Stay audit-ready</h4>
					<p class="mb-3 text-sm text-text-secondary">
						24/7 Access instant, tamper-proof, and audit-ready records stored securely in the cloud.
						Breeze through your next inspection with complete, organised digital logs at your
						fingertips.
					</p>
				</div>

				<div class="rounded bg-bg-secondary p-6 shadow-sm">
					<h4 class="mb-2 text-lg font-semibold text-text-primary">Optimize your workflow</h4>
					<p class="mb-3 text-sm text-text-secondary">
						Stop wasting time on manual paperwork. Our digital system frees up your team to focus on
						what matters—quality service. Streamline daily checks, reduce errors, and improve
						operational efficiency.
					</p>
				</div>

				<div class="rounded bg-bg-secondary p-6 shadow-sm">
					<h4 class="mb-2 text-lg font-semibold text-text-primary">
						AI-powered templates
					</h4>
					<p class="mb-3 text-sm text-text-secondary">
						Get started in seconds. Use our built-in AI to generate custom templates from plain English, or start with pre-loaded, regulatory-standard templates for all your food safety needs and customize them easily.
					</p>
				</div>

				<div class="rounded bg-bg-secondary p-6 shadow-sm">
					<h4 class="mb-2 text-lg font-semibold text-text-primary">Go green, go digital</h4>
					<p class="mb-3 text-sm text-text-secondary">
						Make a positive environmental impact by eliminating paper waste. Our digital solution is
						not only good for your business, but it's also good for the planet.
					</p>
				</div>
			</div>
			<div class="mt-12 grid gap-8 lg:grid-cols-2">
				<div class="rounded bg-bg-secondary p-6 shadow-sm">
					<h3 class="mb-3 text-2xl font-bold text-text-primary">How it works</h3>
					<ul class="space-y-3 text-sm text-text-secondary">
						<li>
							<span class="font-semibold text-text-primary">1. Pick a template:</span> Choose from pre-built
							food safety, cleaning, and temperature templates, or generate one with AI.
						</li>
						<li>
							<span class="font-semibold text-text-primary">2. Assign and remind:</span> Set schedules,
							owners, and reminders so checks never slip.
						</li>
						<li>
							<span class="font-semibold text-text-primary">3. Capture and store:</span> Team members
							log data on any device, stored securely in the cloud.
						</li>
						<li>
							<span class="font-semibold text-text-primary">4. Review and export:</span> Managers audit
							completion, track trends, and export reports for inspectors.
						</li>
					</ul>
				</div>
				<div class="rounded bg-bg-secondary p-6 shadow-sm">
					<h3 class="mb-3 text-2xl font-bold text-text-primary">Generate templates with AI</h3>
					<p class="mb-4 text-sm text-text-secondary">
						Describe your logging needs in plain English, and our AI will instantly generate a custom template tailored to your operations.
					</p>
					<ul class="space-y-2 text-sm text-text-secondary">
						<li class="flex items-start gap-2">
							<span class="font-semibold text-text-primary">✓</span>
							<span>Generate in seconds from natural language</span>
						</li>
						<li class="flex items-start gap-2">
							<span class="font-semibold text-text-primary">✓</span>
							<span>Customize layout with drag-and-drop components</span>
						</li>
						<li class="flex items-start gap-2">
							<span class="font-semibold text-text-primary">✓</span>
							<span>Combine with pre-built templates</span>
						</li>
					</ul>
				</div>
			</div>
			<div class="mt-12 grid gap-8 lg:grid-cols-2">
				<div class="rounded bg-bg-secondary p-6 shadow-sm">
					<h3 class="mb-3 text-2xl font-bold text-text-primary">Who it is for</h3>
					<div class="grid gap-3 md:grid-cols-2">
						<div class="rounded border border-border-primary bg-bg-primary p-4">
							<h4 class="text-lg font-semibold text-text-primary">Restaurants</h4>
							<p class="text-sm text-text-secondary">
								Keep HACCP logs current across shifts and locations.
							</p>
						</div>
						<div class="rounded border border-border-primary bg-bg-primary p-4">
							<h4 class="text-lg font-semibold text-text-primary">Hotels and catering</h4>
							<p class="text-sm text-text-secondary">
								Standardise back-of-house checks for banquets and room service.
							</p>
						</div>
						<div class="rounded border border-border-primary bg-bg-primary p-4">
							<h4 class="text-lg font-semibold text-text-primary">Education</h4>
							<p class="text-sm text-text-secondary">
								Run consistent compliance in cafeterias without paper binders.
							</p>
						</div>
						<div class="rounded border border-border-primary bg-bg-primary p-4">
							<h4 class="text-lg font-semibold text-text-primary">Franchises</h4>
							<p class="text-sm text-text-secondary">
								Roll out templates network-wide and monitor completion in real time.
							</p>
						</div>
					</div>
				</div>
			</div>
			<div class="mt-12 grid gap-6 md:grid-cols-2">
				<div class="rounded bg-bg-secondary p-6 shadow-sm">
					<h3 class="mb-3 text-2xl font-bold text-text-primary">What you get</h3>
					<ul class="space-y-2 text-sm text-text-secondary">
						<li>Real-time completion dashboards and alerts.</li>
						<li>Offline-friendly logging for low-connectivity kitchens.</li>
						<li>Role-based access so the right people see the right logs.</li>
						<li>PDF and CSV exports for auditors and management.</li>
						<li>Secure storage with backups and permissions.</li>
					</ul>
				</div>
				<div class="rounded bg-bg-secondary p-6 shadow-sm">
					<h3 class="mb-3 text-2xl font-bold text-text-primary">Frequently asked questions</h3>
					<dl class="space-y-4 text-sm text-text-secondary">
						<div>
							<dt class="font-semibold text-text-primary">How fast can we go live?</dt>
							<dd>Most teams launch in under a week using the pre-built templates.</dd>
						</div>
						<div>
							<dt class="font-semibold text-text-primary">Do you support multiple sites?</dt>
							<dd>Yes. Manage locations centrally and view completion by site.</dd>
						</div>
						<div>
							<dt class="font-semibold text-text-primary">What about inspections?</dt>
							<dd>Share digital reports or exports with inspectors to speed up audits.</dd>
						</div>
					</dl>
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
