<script lang="ts">
	import TemperaturePicker from '$lib/components/temperature_picker.svelte';
	import UserCheckbox from '$lib/components/user_checkbox.svelte';
	import UserDropdown from '$lib/components/user_dropdown.svelte';
	import UserTextInput from '$lib/components/user_text_input.svelte';
	import UserTextLabel from '$lib/components/user_text_label.svelte';
	import type { CanvasItem } from './types';

	let {
		canvasItems = $bindable(),
		logTitle = $bindable(),
		selectedItemId = $bindable(),
		onExport,
		onDeleteSelected
	}: {
		canvasItems: CanvasItem[];
		logTitle: string;
		selectedItemId: string | null;
		onExport: () => void;
		onDeleteSelected: () => void;
	} = $props();

	let canvasRef: HTMLDivElement;
	let isDraggingExisting = $state(false);
	let dragOffset = $state({ x: 0, y: 0 });
	let draggedComponentType = $state<string | null>(null);

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

	function generateId() {
		return Math.random().toString(36).substring(2, 9);
	}

	let draggedItemId = $state<string | null>(null);

	function handleExistingDragStart(e: DragEvent, item: CanvasItem) {
		isDraggingExisting = true;
		selectedItemId = item.id;
		draggedItemId = item.id;
		draggedComponentType = item.type;

		const target = e.currentTarget as HTMLElement;
		const rect = target.getBoundingClientRect();
		dragOffset = {
			x: e.clientX - rect.left,
			y: e.clientY - rect.top
		};

		if (e.dataTransfer) {
			e.dataTransfer.effectAllowed = 'move';
			e.dataTransfer.setDragImage(target, dragOffset.x, dragOffset.y);
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

		if (isDraggingExisting && draggedItemId) {
			// Direct mutation for better performance - find and update in place
			const itemIndex = canvasItems.findIndex((item) => item.id === draggedItemId);
			if (itemIndex !== -1) {
				canvasItems[itemIndex].x = Math.max(0, x);
				canvasItems[itemIndex].y = Math.max(0, y);
			}
		} else {
			const type = e.dataTransfer?.getData('component-type');
			if (type) {
				const newItem: CanvasItem = {
					id: generateId(),
					type,
					x: Math.max(0, x),
					y: Math.max(0, y),
					props: getDefaultProps(type)
				};
				canvasItems.push(newItem);
				selectedItemId = newItem.id;
			}
		}

		draggedComponentType = null;
		draggedItemId = null;
		isDraggingExisting = false;
	}

	function selectItem(id: string) {
		selectedItemId = id;
	}

	function handleCanvasClick(e: MouseEvent) {
		if (e.target === canvasRef) {
			selectedItemId = null;
		}
	}
</script>

<div class="flex-1 overflow-auto p-6">
	<div class="mx-auto max-w-4xl">
		<div class="mb-4 flex items-center justify-between">
			<h2 class="text-3xl font-bold" style="color: var(--text-secondary);">Canvas</h2>
			<div class="flex gap-2">
				{#if selectedItemId}
					<button
						class="rounded px-4 py-2 font-medium text-white"
						style="background-color: #D9534F;"
						onclick={onDeleteSelected}
					>
						Delete Selected
					</button>
				{/if}
				<button
					class="rounded px-4 py-2 font-medium text-white"
					style="background-color: #337AB7;"
					onclick={onExport}
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
					if (e.key === 'Delete' || e.key === 'Backspace') onDeleteSelected();
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
								placeholder={item.props.placeholder}
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
