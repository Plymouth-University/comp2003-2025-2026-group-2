<script lang="ts">
	import { draggable } from '@neodrag/svelte';
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
		canvasRef = $bindable(),
		onSave,
		onDeleteSelected,
		saving = false,
		saveError = null,
		saveSuccess = false,
		loading = false
	}: {
		canvasItems: CanvasItem[];
		logTitle: string;
		selectedItemId: string | null;
		canvasRef: HTMLDivElement | null;
		onSave: () => void;
		onDeleteSelected: () => void;
		saving?: boolean;
		saveError?: string | null;
		saveSuccess?: boolean;
		loading?: boolean;
	} = $props();

	const SNAP_THRESHOLD = 10;
	let snapLines = $state<{ x: number[]; y: number[] }>({ x: [], y: [] });
	let isDragging = $state(false);
	let snapEnabled = $state(true);

	function getItemBounds(itemId: string): {
		left: number;
		right: number;
		top: number;
		bottom: number;
		centerX: number;
		centerY: number;
		width: number;
		height: number;
	} | null {
		const el = document.querySelector(`[data-item-id="${itemId}"]`) as HTMLElement;
		if (!el || !canvasRef) return null;
		const canvasRect = canvasRef.getBoundingClientRect();
		const itemRect = el.getBoundingClientRect();
		const left = itemRect.left - canvasRect.left;
		const top = itemRect.top - canvasRect.top;
		const right = left + itemRect.width;
		const bottom = top + itemRect.height;
		return {
			left,
			right,
			top,
			bottom,
			width: itemRect.width,
			height: itemRect.height,
			centerX: left + itemRect.width / 2,
			centerY: top + itemRect.height / 2
		};
	}

	function calculateSnap(
		draggedId: string,
		proposedX: number,
		proposedY: number
	): { x: number; y: number; snapLinesX: number[]; snapLinesY: number[] } {
		const draggedEl = document.querySelector(`[data-item-id="${draggedId}"]`) as HTMLElement;
		if (!draggedEl || !canvasRef)
			return { x: proposedX, y: proposedY, snapLinesX: [], snapLinesY: [] };

		const draggedRect = draggedEl.getBoundingClientRect();
		const width = draggedRect.width;
		const height = draggedRect.height;

		const draggedLeft = proposedX;
		const draggedRight = proposedX + width;
		const draggedCenterX = proposedX + width / 2;
		const draggedTop = proposedY;
		const draggedBottom = proposedY + height;
		const draggedCenterY = proposedY + height / 2;

		let bestSnapX: { snappedX: number; distance: number; line: number } | null = null;
		let bestSnapY: { snappedY: number; distance: number; line: number } | null = null;
		const snapLinesX: number[] = [];
		const snapLinesY: number[] = [];

		for (const item of canvasItems) {
			if (item.id === draggedId) continue;
			const bounds = getItemBounds(item.id);
			if (!bounds) continue;

			const xPoints = [
				{ dragPoint: draggedLeft, snapTo: bounds.left, line: bounds.left },
				{ dragPoint: draggedLeft, snapTo: bounds.right, line: bounds.right },
				{ dragPoint: draggedRight, snapTo: bounds.left, line: bounds.left },
				{ dragPoint: draggedRight, snapTo: bounds.right, line: bounds.right },
				{ dragPoint: draggedCenterX, snapTo: bounds.centerX, line: bounds.centerX },
				{ dragPoint: draggedLeft, snapTo: bounds.centerX, line: bounds.centerX },
				{ dragPoint: draggedRight, snapTo: bounds.centerX, line: bounds.centerX },
				{ dragPoint: draggedCenterX, snapTo: bounds.left, line: bounds.left },
				{ dragPoint: draggedCenterX, snapTo: bounds.right, line: bounds.right }
			];

			for (const { dragPoint, snapTo, line } of xPoints) {
				const distance = Math.abs(dragPoint - snapTo);
				if (distance < SNAP_THRESHOLD) {
					if (!bestSnapX || distance < bestSnapX.distance) {
						const offset = dragPoint - draggedLeft;
						bestSnapX = { snappedX: snapTo - offset, distance, line };
					}
				}
			}

			const yPoints = [
				{ dragPoint: draggedTop, snapTo: bounds.top, line: bounds.top },
				{ dragPoint: draggedTop, snapTo: bounds.bottom, line: bounds.bottom },
				{ dragPoint: draggedBottom, snapTo: bounds.top, line: bounds.top },
				{ dragPoint: draggedBottom, snapTo: bounds.bottom, line: bounds.bottom },
				{ dragPoint: draggedCenterY, snapTo: bounds.centerY, line: bounds.centerY },
				{ dragPoint: draggedTop, snapTo: bounds.centerY, line: bounds.centerY },
				{ dragPoint: draggedBottom, snapTo: bounds.centerY, line: bounds.centerY },
				{ dragPoint: draggedCenterY, snapTo: bounds.top, line: bounds.top },
				{ dragPoint: draggedCenterY, snapTo: bounds.bottom, line: bounds.bottom }
			];

			for (const { dragPoint, snapTo, line } of yPoints) {
				const distance = Math.abs(dragPoint - snapTo);
				if (distance < SNAP_THRESHOLD) {
					if (!bestSnapY || distance < bestSnapY.distance) {
						const offset = dragPoint - draggedTop;
						bestSnapY = { snappedY: snapTo - offset, distance, line };
					}
				}
			}
		}

		const finalX = bestSnapX ? bestSnapX.snappedX : proposedX;
		const finalY = bestSnapY ? bestSnapY.snappedY : proposedY;

		if (bestSnapX) snapLinesX.push(bestSnapX.line);
		if (bestSnapY) snapLinesY.push(bestSnapY.line);

		return { x: finalX, y: finalY, snapLinesX, snapLinesY };
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
				<button
					class="rounded px-4 py-2 font-medium"
					class:btn-snap-on={snapEnabled}
					class:btn-snap-off={!snapEnabled}
					onclick={() => (snapEnabled = !snapEnabled)}
				>
					{snapEnabled ? '🧲 Snap On' : '🧲 Snap Off'}
				</button>
				{#if selectedItemId}
					<button
						class="btn-delete rounded px-4 py-2 font-medium text-white"
						onclick={onDeleteSelected}
					>
						Delete Selected
					</button>
				{/if}
				<button
					class="btn-save rounded px-4 py-2 font-medium text-white disabled:opacity-50"
					onclick={onSave}
					disabled={saving || loading}
				>
					{#if saving}
						Saving...
					{:else}
						Save Template
					{/if}
				</button>
			</div>
			{#if saveError}
				<div class="mt-2 text-sm text-red-600">{saveError}</div>
			{/if}
			{#if saveSuccess}
				<div class="mt-2 text-sm text-green-600">Template saved successfully!</div>
			{/if}
		</div>

		{#if loading}
			<div class="flex items-center justify-center py-8">
				<div class="text-lg" style="color: var(--text-secondary);">Loading template...</div>
			</div>
		{:else}
		<div
			class="rounded-lg border-2 p-4"
			style="border-color: var(--border-primary); background-color: var(--bg-primary);"
		>
			<div class="mb-4">
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
				data-canvas
				class="relative min-h-[500px] rounded border-2 border-dashed"
				style="border-color: var(--border-secondary); background-color: var(--bg-secondary);"
				onclick={handleCanvasClick}
				onkeydown={(e) => {
					const target = e.target as HTMLElement;
					const isEditing =
						target.isContentEditable || target.tagName === 'INPUT' || target.tagName === 'TEXTAREA';
					if (e.key === 'Escape') selectedItemId = null;
					if ((e.key === 'Delete' || e.key === 'Backspace') && !isEditing) onDeleteSelected();
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
						data-item-id={item.id}
						class="canvas-item absolute cursor-move rounded bg-white p-2"
						class:border-2={selectedItemId === item.id}
						class:ring-2={selectedItemId === item.id}
						class:ring-blue-500={selectedItemId === item.id}
						class:selected-item={selectedItemId === item.id}
						style="left: {item.x}px; top: {item.y}px; transform: none !important;"
						use:draggable={{
							position: { x: item.x, y: item.y },
							bounds: canvasRef,
							axis:
								item.lockX && item.lockY ? undefined : item.lockX ? 'y' : item.lockY ? 'x' : 'both',
							disabled: item.lockX && item.lockY,
							transform: () => '',
							onDragStart: () => {
								isDragging = true;
							},
							onDrag: ({ offsetX, offsetY }) => {
								const idx = canvasItems.findIndex((i) => i.id === item.id);
								if (idx !== -1) {
									if (snapEnabled) {
										const snap = calculateSnap(item.id, offsetX, offsetY);
										snapLines = { x: snap.snapLinesX, y: snap.snapLinesY };
										if (!item.lockX) canvasItems[idx].x = snap.x;
										if (!item.lockY) canvasItems[idx].y = snap.y;
									} else {
										snapLines = { x: [], y: [] };
										if (!item.lockX) canvasItems[idx].x = offsetX;
										if (!item.lockY) canvasItems[idx].y = offsetY;
									}
								}
							},
							onDragEnd: () => {
								isDragging = false;
								snapLines = { x: [], y: [] };
							}
						}}
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
								bind:text={item.props.text}
								size={item.props.size}
								weight={item.props.weight}
							/>
						{/if}
					</div>
				{/each}

				{#if isDragging}
					{#each snapLines.x as lineX}
						<div
							class="snap-line-vertical pointer-events-none absolute top-0 bottom-0 w-px"
							style="left: {lineX}px; background-color: #3b82f6;"
						></div>
					{/each}
					{#each snapLines.y as lineY}
						<div
							class="snap-line-horizontal pointer-events-none absolute right-0 left-0 h-px"
							style="top: {lineY}px; background-color: #3b82f6;"
						></div>
					{/each}
				{/if}
			</div>
		</div>
		{/if}
	</div>
</div>

<style>
	.canvas-item {
		touch-action: none;
	}

	.selected-item {
		border-color: var(--border-primary);
	}

	.snap-line-vertical,
	.snap-line-horizontal {
		z-index: 1000;
		opacity: 0.8;
	}

	.btn-delete {
		background-color: #d9534f;
		transition: background-color 0.15s ease;
	}
	.btn-delete:hover {
		background-color: #c9302c;
	}
	.btn-delete:active {
		background-color: #ac2925;
	}

	.btn-save {
		background-color: #337ab7;
		transition: background-color 0.15s ease;
	}
	.btn-save:hover:not(:disabled) {
		background-color: #286090;
	}
	.btn-save:active:not(:disabled) {
		background-color: #204d74;
	}

	.btn-snap-on {
		background-color: #5cb85c;
		color: white;
		transition: background-color 0.15s ease;
	}
	.btn-snap-on:hover {
		background-color: #449d44;
	}
	.btn-snap-on:active {
		background-color: #398439;
	}

	.btn-snap-off {
		background-color: #6c757d;
		color: white;
		transition: background-color 0.15s ease;
	}
	.btn-snap-off:hover {
		background-color: #5a6268;
	}
	.btn-snap-off:active {
		background-color: #4e555b;
	}
</style>
