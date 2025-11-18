<script lang="ts">
	// Sample template data
	const templates = [
		{ id: 1, name: 'Kitchen Daily Log', selected: false },
		{ id: 2, name: 'Kitchen Cleaning Log', selected: false },
		{ id: 3, name: 'Bar Log', selected: false }
	];

	// Sample components
	const components = [
		{ id: 1, name: 'Text Input', icon: 'T' },
		{ id: 2, name: 'Checkbox', icon: '✓' },
		{ id: 3, name: 'Temperature Input', icon: '🌡' },
		{ id: 4, name: 'Dropdown', icon: '≡' }
	];

	// Sample canvas items
	let canvasItems = [
		{ id: 1, value: 3, label: '°C Fridge 1' },
		{ id: 2, value: 3, label: '°C Fridge 2' },
		{ id: 3, value: 3, label: '°C Freezer 1' }
	];
</script>

<div class="min-h-screen" style="background-color: #F8F8F8;">
	<!-- Navigation Header -->
	<div class="border-b-2" style="border-color: #000100;">
		<div class="mx-auto max-w-7xl px-6 py-4" style="background-color: #B4D2E7;">
			<div class="flex items-center justify-end">
				<!-- Navigation Menu -->
				<nav class="flex items-center gap-6">
					<a href="/dashboard" class="hover:opacity-80" style="color: #000100;">
						Dashboard
					</a>
					<a href="/logs" class="hover:opacity-80" style="color: #000100;">
						Logs
					</a>
					<a href="/users" class="hover:opacity-80" style="color: #000100;">
						Users
					</a>
					<a href="/templates-dashboard" class="font-medium border-b-2 pb-1" style="color: #000100; border-color: #000100;">
						Templates
					</a>
					<a href="/reports" class="hover:opacity-80" style="color: #000100;">
						Reports
					</a>
				</nav>
			</div>
		</div>
	</div>

	<!-- Main Content - Three Column Layout -->
	<div class="flex h-[calc(100vh-73px)]">
		<!-- Left Sidebar - Templates -->
		<div class="w-64 border-r-2 bg-white" style="border-color: #000100;">
			<div class="p-6">
				<h2 class="text-2xl font-bold mb-4" style="color: #000100;">Templates</h2>
				
				<button class="w-full px-4 py-2 mb-6 text-white font-medium rounded" style="background-color: #5CB85C;">
					+ Create New
				</button>

				<div class="border-2" style="border-color: #000100;">
					<ul class="divide-y" style="border-color: #000100;">
						{#each templates as template}
							<li class="flex items-center gap-3 px-4 py-3 hover:bg-gray-50">
								<div class="w-5 h-5 border-2 flex items-center justify-center" style="border-color: #000100;">
									{#if template.selected}
										<svg width="12" height="12" viewBox="0 0 12 12" fill="none" stroke="#000100" stroke-width="2">
											<line x1="2" y1="6" x2="10" y2="6"></line>
											<line x1="6" y1="2" x2="6" y2="10"></line>
										</svg>
									{/if}
								</div>
								<span style="color: #000100;">{template.name}</span>
							</li>
						{/each}
					</ul>
				</div>
			</div>
		</div>

		<!-- Middle Section - Canvas -->
		<div class="flex-1 p-6 overflow-auto">
			<div class="max-w-2xl mx-auto">
				<h2 class="text-3xl font-bold text-center mb-6" style="color: #A1A6B4;">Canvas</h2>
				
				<div class="border-2 rounded-lg bg-white p-6" style="border-color: #000100;">
					<h3 class="text-xl font-bold mb-4" style="color: #000100;">New Log Template</h3>
					
					<div class="mb-6">
						<label for="log-title-input" class="block text-lg font-bold mb-2" style="color: #A1A6B4;">Log Title</label>
						<input
							id="log-title-input"
							type="text" 
							placeholder="Description"
							class="w-full px-4 py-2 border-2"
							style="border-color: #000100;"
						/>
					</div>

					<div class="space-y-3">
						{#each canvasItems as item}
							<div class="flex items-center gap-3">
								<input 
									type="number" 
									bind:value={item.value}
									class="w-16 px-3 py-2 text-center border-2"
									style="border-color: #000100;"
								/>
								<span style="color: #000100;">{item.label}</span>
							</div>
						{/each}
					</div>
				</div>
			</div>
		</div>

		<!-- Right Sidebar - Components -->
		<div class="w-64 border-l-2 bg-white" style="border-color: #000100;">
			<div class="p-6">
				<h2 class="text-2xl font-bold mb-6" style="color: #000100;">Components</h2>
				
				<div class="space-y-3">
					{#each components as component}
						<div class="border-2 px-4 py-3 flex items-center gap-3 hover:bg-gray-50 cursor-pointer" style="border-color: #000100;">
							<div class="w-8 h-8 border-2 flex items-center justify-center font-bold" style="border-color: #000100; color: #000100;">
								{component.icon}
							</div>
							<span style="color: #000100;">{component.name}</span>
						</div>
					{/each}
				</div>
			</div>
		</div>
	</div>
</div>
