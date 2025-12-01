<script lang="ts">
	import { browser } from '$app/environment';
	import TemperaturePicker from '$lib/components/temperature_picker.svelte';
	import UserCheckbox from '$lib/components/user_checkbox.svelte';
	import UserDropdown from '$lib/components/user_dropdown.svelte';
	import UserTextInput from '$lib/components/user_text_input.svelte';
	import UserTextLabel from '$lib/components/user_text_label.svelte';

	const templates = [
		{ id: 1, name: 'Kitchen Daily Log', selected: false },
		{ id: 2, name: 'Kitchen Cleaning Log', selected: false },
		{ id: 3, name: 'Bar Log', selected: false }
	];

	const componentTypes = [
		{ type: 'text_input', name: 'Text Input', icon: 'T' },
		{ type: 'checkbox', name: 'Checkbox', icon: '✓' },
		{ type: 'temperature', name: 'Temperature Input', icon: '🌡' },
		{ type: 'dropdown', name: 'Dropdown', icon: '≡' },
		{ type: 'label', name: 'Text Label', icon: 'L' }
	];

	type CanvasItem = {
		id: string;
		type: string;
		x: number;
		y: number;
		props: Record<string, any>;
	};

	let canvasItems = $state<CanvasItem[]>(
		browser && localStorage.getItem('canvasItems')
			? JSON.parse(localStorage.getItem('canvasItems')!)
			: []
	);

	let logTitle = $state(
		browser && localStorage.getItem('logTitle') ? localStorage.getItem('logTitle')! : ''
	);

	let selectedItemId = $state<string | null>(null);
	let draggedComponentType = $state<string | null>(null);
	let isDraggingExisting = $state(false);
	let dragOffset = $state({ x: 0, y: 0 });
	let canvasRef: HTMLDivElement;

	$effect(() => {
		if (browser) {
			localStorage.setItem('canvasItems', JSON.stringify(canvasItems));
		}
	});

	$effect(() => {
		if (browser) {
			localStorage.setItem('logTitle', logTitle);
		}
	});

	function createNewTemplate() {
		canvasItems = [];
		logTitle = '';
		selectedItemId = null;
	}

	function generateId() {
		return Math.random().toString(36).substring(2, 9);
	}

	function getDefaultProps(type: string): Record<string, any> {
		switch (type) {
			case 'text_input':
				return { text: '', size: 'medium', weight: 'normal' };
			case 'checkbox':
				return { text: 'Checkbox Label', size: '16px', weight: 'normal' };
			case 'temperature':
				return { value: 0, min: -20, max: 50, label: 'Temperature', unit: '°C' };
			case 'dropdown':
				return { selected: '', options: ['Option 1', 'Option 2', 'Option 3'] };
			case 'label':
				return { editable: true, text: 'Label Text', size: 'medium', weight: 'normal' };
			default:
				return {};
		}
	}

	function handleDragStart(e: DragEvent, type: string) {
		draggedComponentType = type;
		isDraggingExisting = false;
		if (e.dataTransfer) {
			e.dataTransfer.effectAllowed = 'copy';
		}
	}

	function handleExistingDragStart(e: DragEvent, item: CanvasItem) {
		isDraggingExisting = true;
		selectedItemId = item.id;
		draggedComponentType = item.type;

		const rect = (e.target as HTMLElement).getBoundingClientRect();
		dragOffset = {
			x: e.clientX - rect.left,
			y: e.clientY - rect.top
		};

		if (e.dataTransfer) {
			e.dataTransfer.effectAllowed = 'move';
		}
	}

	function handleDragOver(e: DragEvent) {
		e.preventDefault();
		if (e.dataTransfer) {
			e.dataTransfer.dropEffect = isDraggingExisting ? 'move' : 'copy';
		}
	}

	function handleDrop(e: DragEvent) {
		e.preventDefault();

		if (!canvasRef) return;

		const rect = canvasRef.getBoundingClientRect();
		const x = e.clientX - rect.left - (isDraggingExisting ? dragOffset.x : 0);
		const y = e.clientY - rect.top - (isDraggingExisting ? dragOffset.y : 0);

		if (isDraggingExisting && selectedItemId) {
			canvasItems = canvasItems.map((item) =>
				item.id === selectedItemId ? { ...item, x: Math.max(0, x), y: Math.max(0, y) } : item
			);
		} else if (draggedComponentType) {
			const newItem: CanvasItem = {
				id: generateId(),
				type: draggedComponentType,
				x: Math.max(0, x),
				y: Math.max(0, y),
				props: getDefaultProps(draggedComponentType)
			};
			canvasItems = [...canvasItems, newItem];
			selectedItemId = newItem.id;
		}

		draggedComponentType = null;
		isDraggingExisting = false;
	}

	function selectItem(id: string) {
		selectedItemId = id;
	}

	function deleteSelected() {
		if (selectedItemId) {
			canvasItems = canvasItems.filter((item) => item.id !== selectedItemId);
			selectedItemId = null;
		}
	}

	function handleCanvasClick(e: MouseEvent) {
		if (e.target === canvasRef) {
			selectedItemId = null;
		}
	}

	function exportToJson() {
		const exportData = {
			title: logTitle,
			items: canvasItems.map((item) => ({
				type: item.type,
				x: item.x,
				y: item.y,
				props: item.props
			}))
		};
		console.log('Exported Template JSON:', JSON.stringify(exportData, null, 2));
	}

	function updateItemProp(itemId: string, propKey: string, value: any) {
		canvasItems = canvasItems.map((item) =>
			item.id === itemId ? { ...item, props: { ...item.props, [propKey]: value } } : item
		);
	}

	let selectedItem = $derived(canvasItems.find((item) => item.id === selectedItemId));
</script>

<div class="min-h-full" style="background-color: var(--bg-secondary);">
	<div class="flex h-[calc(100vh-73px)]">
		<div
			class="w-64 border-r-2"
			style="border-color: var(--border-primary); background-color: var(--bg-primary);"
		>
			<div class="p-6">
				<h2 class="mb-4 text-2xl font-bold" style="color: var(--text-primary);">Templates</h2>

				<button
					class="mb-6 w-full rounded px-4 py-2 font-medium text-white"
					style="background-color: #5CB85C;"
					onclick={createNewTemplate}
				>
					+ Create New
				</button>

				<div class="border-2" style="border-color: var(--border-primary);">
					<ul class="divide-y" style="border-color: var(--border-secondary);">
						{#each templates as template (template.id)}
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

		<div class="flex-1 overflow-auto p-6">
			<div class="mx-auto max-w-4xl">
				<div class="mb-4 flex items-center justify-between">
					<h2 class="text-3xl font-bold" style="color: var(--text-secondary);">Canvas</h2>
					<div class="flex gap-2">
						{#if selectedItemId}
							<button
								class="rounded px-4 py-2 font-medium text-white"
								style="background-color: #D9534F;"
								onclick={deleteSelected}
							>
								Delete Selected
							</button>
						{/if}
						<button
							class="rounded px-4 py-2 font-medium text-white"
							style="background-color: #337AB7;"
							onclick={exportToJson}
						>
							Export JSON
						</button>
					</div>
				</div>

				<div
					class="rounded-lg border-2 p-4"
					style="border-color: var(--border-primary); background-color: var(--bg-primary);"
				>
					<div class="mb-4">
						<label
							for="log-title-input"
							class="mb-2 block text-lg font-bold"
							style="color: var(--text-secondary);">Log Title</label
						>
						<input
							id="log-title-input"
							type="text"
							bind:value={logTitle}
							placeholder="Enter template title..."
							class="w-full border-2 px-4 py-2"
							style="border-color: var(--border-primary); background-color: var(--bg-primary); color: var(--text-primary);"
						/>
					</div>

					<!-- svelte-ignore a11y_no_noninteractive_element_interactions -->
					<!-- svelte-ignore a11y_no_noninteractive_tabindex -->
					<div
						bind:this={canvasRef}
						class="relative min-h-[500px] rounded border-2 border-dashed"
						style="border-color: var(--border-secondary); background-color: var(--bg-secondary);"
						ondragover={handleDragOver}
						ondrop={handleDrop}
						onclick={handleCanvasClick}
						onkeydown={(e) => {
							if (e.key === 'Escape') selectedItemId = null;
							if (e.key === 'Delete' || e.key === 'Backspace') deleteSelected();
						}}
						role="application"
						tabindex="0"
						aria-label="Design canvas - drag components here to design your template"
					>
						{#if canvasItems.length === 0}
							<div class="pointer-events-none absolute inset-0 flex items-center justify-center">
								<p class="text-lg opacity-50" style="color: var(--text-secondary);">
									Drag components here to start designing
								</p>
							</div>
						{/if}

						{#each canvasItems as item (item.id)}
							<div
								class="absolute cursor-move rounded border-2 bg-white p-2"
								class:ring-2={selectedItemId === item.id}
								class:ring-blue-500={selectedItemId === item.id}
								style="left: {item.x}px; top: {item.y}px; border-color: var(--border-primary);"
								draggable="true"
								ondragstart={(e) => handleExistingDragStart(e, item)}
								onclick={(e) => {
									e.stopPropagation();
									selectItem(item.id);
								}}
								onkeydown={(e) => {
									if (e.key === 'Enter') selectItem(item.id);
								}}
								role="button"
								tabindex="0"
							>
								{#if item.type === 'text_input'}
									<UserTextInput
										text={item.props.text}
										size={item.props.size}
										weight={item.props.weight}
									/>
								{:else if item.type === 'checkbox'}
									<UserCheckbox
										text={item.props.text}
										size={item.props.size}
										weight={item.props.weight}
									/>
								{:else if item.type === 'temperature'}
									<TemperaturePicker
										bind:value={item.props.value}
										min={item.props.min}
										max={item.props.max}
										label={item.props.label}
										unit={item.props.unit}
									/>
								{:else if item.type === 'dropdown'}
									<UserDropdown bind:selected={item.props.selected} options={item.props.options} />
								{:else if item.type === 'label'}
									<UserTextLabel
										editable={item.props.editable}
										text={item.props.text}
										size={item.props.size}
										weight={item.props.weight}
									/>
								{/if}
							</div>
						{/each}
					</div>
				</div>
			</div>
		</div>

		<div
			class="flex w-72 flex-col border-l-2"
			style="border-color: var(--border-primary); background-color: var(--bg-primary);"
		>
			<div class="border-b-2 p-6" style="border-color: var(--border-primary);">
				<h2 class="mb-4 text-2xl font-bold" style="color: var(--text-primary);">Components</h2>

				<div class="space-y-3">
					{#each componentTypes as component (component.type)}
						<div
							class="flex cursor-grab items-center gap-3 border-2 px-4 py-3 hover:opacity-80 active:cursor-grabbing"
							style="border-color: var(--border-primary);"
							draggable="true"
							ondragstart={(e) => handleDragStart(e, component.type)}
							role="button"
							tabindex="0"
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

			{#if selectedItem}
				<div class="flex-1 overflow-auto p-6">
					<h3 class="mb-4 text-xl font-bold" style="color: var(--text-primary);">Properties</h3>

					{#if selectedItem.type === 'text_input' || selectedItem.type === 'label'}
						<div class="space-y-4">
							<div>
								<span class="mb-1 block text-sm font-medium" style="color: var(--text-secondary);"
									>Text</span
								>
								<input
									type="text"
									value={selectedItem.props.text}
									oninput={(e) => updateItemProp(selectedItem.id, 'text', e.currentTarget.value)}
									class="w-full border-2 px-3 py-2"
									style="border-color: var(--border-primary); color: var(--text-primary);"
								/>
							</div>
							<div>
								<span class="mb-1 block text-sm font-medium" style="color: var(--text-secondary);"
									>Size</span
								>
								<select
									value={selectedItem.props.size}
									onchange={(e) => updateItemProp(selectedItem.id, 'size', e.currentTarget.value)}
									class="w-full border-2 px-3 py-2"
									style="border-color: var(--border-primary); color: var(--text-primary);"
								>
									<option value="small">Small</option>
									<option value="medium">Medium</option>
									<option value="large">Large</option>
								</select>
							</div>
							<div>
								<span class="mb-1 block text-sm font-medium" style="color: var(--text-secondary);"
									>Weight</span
								>
								<select
									value={selectedItem.props.weight}
									onchange={(e) => updateItemProp(selectedItem.id, 'weight', e.currentTarget.value)}
									class="w-full border-2 px-3 py-2"
									style="border-color: var(--border-primary); color: var(--text-primary);"
								>
									<option value="light">Light</option>
									<option value="normal">Normal</option>
									<option value="bold">Bold</option>
								</select>
							</div>
						</div>
					{:else if selectedItem.type === 'checkbox'}
						<div class="space-y-4">
							<div>
								<span class="mb-1 block text-sm font-medium" style="color: var(--text-secondary);"
									>Label</span
								>
								<input
									type="text"
									value={selectedItem.props.text}
									oninput={(e) => updateItemProp(selectedItem.id, 'text', e.currentTarget.value)}
									class="w-full border-2 px-3 py-2"
									style="border-color: var(--border-primary); color: var(--text-primary);"
								/>
							</div>
						</div>
					{:else if selectedItem.type === 'temperature'}
						<div class="space-y-4">
							<div>
								<span class="mb-1 block text-sm font-medium" style="color: var(--text-secondary);"
									>Label</span
								>
								<input
									type="text"
									value={selectedItem.props.label}
									oninput={(e) => updateItemProp(selectedItem.id, 'label', e.currentTarget.value)}
									class="w-full border-2 px-3 py-2"
									style="border-color: var(--border-primary); color: var(--text-primary);"
								/>
							</div>
							<div>
								<span class="mb-1 block text-sm font-medium" style="color: var(--text-secondary);"
									>Unit</span
								>
								<input
									type="text"
									value={selectedItem.props.unit}
									oninput={(e) => updateItemProp(selectedItem.id, 'unit', e.currentTarget.value)}
									class="w-full border-2 px-3 py-2"
									style="border-color: var(--border-primary); color: var(--text-primary);"
								/>
							</div>
							<div>
								<span class="mb-1 block text-sm font-medium" style="color: var(--text-secondary);"
									>Min</span
								>
								<input
									type="number"
									value={selectedItem.props.min}
									oninput={(e) =>
										updateItemProp(selectedItem.id, 'min', parseInt(e.currentTarget.value))}
									class="w-full border-2 px-3 py-2"
									style="border-color: var(--border-primary); color: var(--text-primary);"
								/>
							</div>
							<div>
								<span class="mb-1 block text-sm font-medium" style="color: var(--text-secondary);"
									>Max</span
								>
								<input
									type="number"
									value={selectedItem.props.max}
									oninput={(e) =>
										updateItemProp(selectedItem.id, 'max', parseInt(e.currentTarget.value))}
									class="w-full border-2 px-3 py-2"
									style="border-color: var(--border-primary); color: var(--text-primary);"
								/>
							</div>
						</div>
					{:else if selectedItem.type === 'dropdown'}
						<div class="space-y-4">
							<div>
								<span class="mb-1 block text-sm font-medium" style="color: var(--text-secondary);"
									>Options (one per line)</span
								>
								<textarea
									value={selectedItem.props.options.join('\n')}
									oninput={(e) =>
										updateItemProp(
											selectedItem.id,
											'options',
											e.currentTarget.value.split('\n').filter((o: string) => o.trim())
										)}
									class="h-32 w-full border-2 px-3 py-2"
									style="border-color: var(--border-primary); color: var(--text-primary);"
								></textarea>
							</div>
						</div>
					{/if}
				</div>
			{/if}
		</div>
	</div>
</div>

<style>
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
