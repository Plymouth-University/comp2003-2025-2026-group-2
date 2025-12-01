<script lang="ts">
	const templates = [
		{ id: 1, name: 'Kitchen Daily Log', selected: false },
		{ id: 2, name: 'Kitchen Cleaning Log', selected: false },
		{ id: 3, name: 'Bar Log', selected: false }
	];

	const components = [
		{ id: 1, name: 'Text Input', icon: 'T' },
		{ id: 2, name: 'Checkbox', icon: '✓' },
		{ id: 3, name: 'Temperature Input', icon: '🌡' },
		{ id: 4, name: 'Dropdown', icon: '≡' }
	];

	let canvasItems = $state([
		{ id: 1, value: 3, label: 'Log 1', unit: '°C' },
		{ id: 2, value: 5, label: 'Log 2', unit: '°C' },
		{ id: 3, value: 4, label: 'Log 3', unit: '°C' }
	]);
</script>

<div class="min-h-full" style="background-color: var(--bg-secondary);">
	<!-- Main Content - Three Column Layout -->
	<div class="flex h-[calc(100vh-73px)]">
		<!-- Left Sidebar - Templates -->
		<div class="w-64 border-r-2" style="border-color: var(--border-primary); background-color: var(--bg-primary);">
			<div class="p-6">
				<h2 class="mb-4 text-2xl font-bold" style="color: var(--text-primary);">Templates</h2>

				<button
					class="mb-6 w-full rounded px-4 py-2 font-medium text-white"
					style="background-color: #5CB85C;"
				>
					+ Create New
				</button>

				<div class="border-2" style="border-color: var(--border-primary);">
					<ul class="divide-y" style="border-color: var(--border-secondary);">
						{#each templates as template}
							<li class="flex items-center gap-3 px-4 py-3 hover:opacity-80">
								<div
									class="flex h-5 w-5 items-center justify-center border-2"
									style="border-color: var(--border-primary);"
								>
									{#if template.selected}
										<svg
											width="12"
											height="12"
											viewBox="0 0 12 12"
											fill="none"
											stroke="currentColor"
											stroke-width="2"
											style="color: var(--border-primary);"
										>
											<line x1="2" y1="6" x2="10" y2="6"></line>
											<line x1="6" y1="2" x2="6" y2="10"></line>
										</svg>
									{/if}
								</div>
								<span style="color: var(--text-primary);">{template.name}</span>
							</li>
						{/each}
					</ul>
				</div>
			</div>
		</div>

		<!-- Middle Section - Canvas -->
		<div class="flex-1 overflow-auto p-6">
			<div class="mx-auto max-w-2xl">
				<h2 class="mb-6 text-center text-3xl font-bold" style="color: var(--text-secondary);">Canvas</h2>

				<div class="rounded-lg border-2 p-8" style="border-color: var(--border-primary); background-color: var(--bg-primary);">
					<h3 class="mb-6 text-xl font-bold" style="color: var(--text-primary);">New Log Template</h3>

					<div class="mb-8">
						<label
							for="log-title-input"
							class="mb-2 block text-lg font-bold"
							style="color: var(--text-secondary);">Log Title</label
						>
						<input
							id="log-title-input"
							type="text"
							placeholder="Description"
							class="w-full border-2 px-4 py-2"
							style="border-color: var(--border-primary); background-color: var(--bg-primary); color: var(--text-primary);"
						/>
					</div>

					<!-- Three Column Header -->
					<div class="mb-4 grid grid-cols-[120px_1fr_120px] gap-8">
						<div class="text-center font-bold" style="color: var(--text-primary);">
							<!-- Empty for input column -->
						</div>
						<div class="text-lg font-bold" style="color: var(--text-primary);">
							names
							<div class="mt-1 h-0.5 bg-current"></div>
						</div>
						<div class="text-right text-lg font-bold" style="color: var(--text-primary);">
							units
							<div class="mt-1 h-0.5 bg-current"></div>
						</div>
					</div>

					<!-- Temperature Input Rows -->
					<div class="space-y-6">
						{#each canvasItems as item}
							<div class="grid grid-cols-[120px_1fr_120px] items-center gap-8">
								<!-- Input with buttons -->
								<div class="flex items-center gap-3">
									<input
										type="number"
										bind:value={item.value}
										class="w-16 border-2 px-3 py-2 text-center text-xl font-medium"
										style="border-color: var(--border-primary); color: var(--text-primary); background-color: var(--bg-primary);"
									/>
									<div class="flex flex-col gap-1">
										<button
											type="button"
											onclick={() => item.value++}
											class="flex h-7 w-7 items-center justify-center rounded-sm border-2 transition-all hover:scale-110 hover:opacity-80 active:scale-95"
											style="border-color: var(--border-primary); color: var(--text-primary);"
											aria-label="Increase temperature"
										>
											<svg
												width="14"
												height="14"
												viewBox="0 0 14 14"
												fill="none"
												stroke="currentColor"
												stroke-width="2.5"
											>
												<polyline points="2 8 7 3 12 8"></polyline>
											</svg>
										</button>
										<button
											type="button"
											onclick={() => item.value--}
											class="flex h-7 w-7 items-center justify-center rounded-sm border-2 transition-all hover:scale-110 hover:opacity-80 active:scale-95"
											style="border-color: var(--border-primary); color: var(--text-primary);"
											aria-label="Decrease temperature"
										>
											<svg
												width="14"
												height="14"
												viewBox="0 0 14 14"
												fill="none"
												stroke="currentColor"
												stroke-width="2.5"
											>
												<polyline points="2 6 7 11 12 6"></polyline>
											</svg>
										</button>
									</div>
								</div>

								<!-- Name/Label -->
								<div class="text-lg" style="color: var(--text-primary);">
									{item.label}
								</div>

								<!-- Units -->
								<div class="text-right text-lg" style="color: var(--text-primary);">
									{item.unit}
								</div>
							</div>
						{/each}
					</div>
				</div>
			</div>
		</div>

		<!-- Right Sidebar - Components -->
		<div class="w-64 border-l-2" style="border-color: var(--border-primary); background-color: var(--bg-primary);">
			<div class="p-6">
				<h2 class="mb-6 text-2xl font-bold" style="color: var(--text-primary);">Components</h2>

				<div class="space-y-3">
					{#each components as component}
						<div
							class="flex cursor-pointer items-center gap-3 border-2 px-4 py-3 hover:opacity-80"
							style="border-color: var(--border-primary);"
						>
							<div
								class="flex h-8 w-8 items-center justify-center border-2 font-bold"
								style="border-color: var(--border-primary); color: var(--text-primary);"
							>
								{component.icon}
							</div>
							<span style="color: var(--text-primary);">{component.name}</span>
						</div>
					{/each}
				</div>
			</div>
		</div>
	</div>
</div>

<style>
	/* Remove default number input arrows */
	input[type='number']::-webkit-inner-spin-button,
	input[type='number']::-webkit-outer-spin-button {
		-webkit-appearance: none;
		margin: 0;
	}

	input[type='number'] {
		-moz-appearance: textfield;
		appearance: textfield;
	}
</style>
