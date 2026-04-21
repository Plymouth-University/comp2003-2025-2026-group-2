<script lang="ts">
	import { calculateSnap } from '$lib/utils/snap';
	import CanvasItemComponent from './CanvasItem.svelte';
	import type { CanvasItem } from './types';

	let {
		canvasItems = $bindable(),
		logTitle = $bindable(),
		versionName = $bindable(),
		selectedItemId = $bindable(),
		canvasRef = $bindable(),
		canvasHeight = $bindable(500),
		branchId = $bindable(null),
		branches = [],
		canManageCompany = false,
		onSave,
		onDeleteSelected,
		onDeleteTemplate,
		onItemMoved,
		onShowHistory,
		saving = false,
		saveError = null,
		saveSuccess = false,
		loading = false,
		isEditing = false,
		deleting = false,
		deleteError = null
	}: {
		canvasItems: CanvasItem[];
		logTitle: string;
		versionName: string;
		selectedItemId: string | null;
		canvasRef: HTMLDivElement | null;
		canvasHeight?: number;
		branchId?: string | null;
		branches?: { id: string; name: string }[];
		canManageCompany?: boolean;
		onSave: () => void;
		onDeleteSelected: () => void;
		onDeleteTemplate: () => void;
		onItemMoved?: () => void;
		onShowHistory?: () => void;
		saving?: boolean;
		saveError?: string | null;
		saveSuccess?: boolean;
		loading?: boolean;
		isEditing?: boolean;
		deleting?: boolean;
		deleteError?: string | null;
	} = $props();

	let snapLines = $state<{ x: number[]; y: number[] }>({ x: [], y: [] });
	let isDragging = $state(false);
	let snapEnabled = $state(true);

	function selectItem(id: string) {
		selectedItemId = id;
	}

	function handleCanvasClick(e: MouseEvent) {
		if (e.target === canvasRef) {
			selectedItemId = null;
		}
	}

	function handleCanvasResizeStart(e: MouseEvent) {
		e.preventDefault();
		const startY = e.clientY;
		const startHeight = canvasHeight;

		const handleMouseMove = (e: MouseEvent) => {
			const delta = e.clientY - startY;
			canvasHeight = Math.max(300, startHeight + delta);
		};

		const handleMouseUp = () => {
			document.removeEventListener('mousemove', handleMouseMove);
			document.removeEventListener('mouseup', handleMouseUp);
		};

		document.addEventListener('mousemove', handleMouseMove);
		document.addEventListener('mouseup', handleMouseUp);
	}
</script>

<div class="flex-1 overflow-auto p-6">
	<div class="mx-auto max-w-4xl">
		<div class="mb-4 flex items-center justify-between">
			<h2 class="text-3xl font-bold text-text-primary">
				Canvas
				<div class="tooltip" tabindex="-1" aria-label="Canvas instructions" role="tooltip">
					﹖
					<span class="tooltip-text"
						>Drag and drop log components onto the canvas to create a log template. Save your log
						template to make it completable.</span
					>
				</div>
			</h2>
			<div class="flex gap-2">
				<button
					class="cursor-pointer rounded px-4 py-2 font-medium"
					class:btn-snap-on={snapEnabled}
					class:btn-snap-off={!snapEnabled}
					onclick={() => (snapEnabled = !snapEnabled)}
				>
					{snapEnabled ? '🧲 Snap On' : '🧲 Snap Off'}
				</button>
				{#if isEditing && onShowHistory}
					<button
						class="btn-history cursor-pointer rounded px-4 py-2 font-medium text-white"
						onclick={() => {
							console.log('History button clicked');
							if (onShowHistory) onShowHistory();
						}}
					>
						🕒 History
					</button>
				{/if}
				{#if selectedItemId}
					<button
						class="btn-delete cursor-pointer rounded px-4 py-2 font-medium text-white"
						onclick={onDeleteSelected}
					>
						Delete Selected
					</button>
				{/if}
				{#if isEditing}
					<button
						class="btn-delete cursor-pointer rounded px-4 py-2 font-medium text-white disabled:opacity-50"
						onclick={onDeleteTemplate}
						disabled={deleting || loading}
					>
						{#if deleting}
							Deleting...
						{:else}
							Delete Template
						{/if}
					</button>
				{/if}
				<button
					class="btn-save cursor-pointer rounded px-4 py-2 font-medium text-white disabled:opacity-50"
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
		</div>
		{#if saveError}
			<div class="mt-2 text-sm text-red-600">{saveError}</div>
		{/if}
		{#if deleteError}
			<div class="mt-2 text-sm text-red-600">{deleteError}</div>
		{/if}
		{#if saveSuccess}
			<div class="mt-2 text-sm text-green-600">Template saved successfully!</div>
		{/if}

		{#if loading}
			<div class="flex items-center justify-center py-8">
				<div class="text-lg text-text-secondary">Loading template...</div>
			</div>
		{:else}
			<div class="rounded-lg border-2 border-border-primary bg-bg-primary p-4">
				<div class="flex">
					<div class="mb-4 grow-6">
						<label for="log-title-input" class="sr-only">Template title</label>
						<input
							id="log-title-input"
							type="text"
							bind:value={logTitle}
							placeholder="Template Name"
							class="w-full border-2 border-border-primary bg-bg-primary px-4 py-2 text-text-primary"
						/>
					</div>

					{#if isEditing}
						<div class="mb-4 grow">
							<label for="version-name-input" class="sr-only">Version name (optional)</label>
							<input
								id="version-name-input"
								type="text"
								bind:value={versionName}
								placeholder="Give this version a name (optional)..."
								class="h-full w-full border-2 border-l-0 border-border-primary bg-bg-primary px-4 py-2 text-sm text-text-primary italic"
							/>
						</div>
					{/if}
					<div class="mb-4 grow">
						<label for="branch-select" class="sr-only">Branch visibility</label>
						<select
							id="branch-select"
							bind:value={branchId}
							class="h-full w-full border-2 border-l-0 border-border-primary bg-bg-primary px-4 py-2 text-sm text-text-primary"
						>
							{#if canManageCompany}
								<option value="company">Company Wide Visibility</option>
							{/if}
							{#each branches as branch (branch.id)}
								<option value={branch.id}>{branch.name}</option>
							{/each}
						</select>
					</div>
				</div>

				<!-- svelte-ignore a11y_no_noninteractive_element_interactions -->
				<!-- svelte-ignore a11y_no_noninteractive_tabindex -->
				<div class="relative">
					<div
						bind:this={canvasRef}
						data-canvas
						class="relative rounded border-2 border-dashed border-border-secondary bg-bg-secondary"
						style="height: {canvasHeight}px;"
						onclick={handleCanvasClick}
						onkeydown={(e) => {
							const target = e.target as HTMLElement;
							const isEditing =
								target.isContentEditable ||
								target.tagName === 'INPUT' ||
								target.tagName === 'TEXTAREA';
							if (e.key === 'Escape') selectedItemId = null;
							if ((e.key === 'Delete' || e.key === 'Backspace') && !isEditing) onDeleteSelected();
						}}
						role="application"
						tabindex="0"
						aria-label="Design canvas - drag components here to design your template"
					>
						{#if canvasItems.length === 0}
							<div class="pointer-events-none absolute inset-0 flex items-center justify-center">
								<p class="text-lg text-text-secondary opacity-50">
									Drag components here to start designing
								</p>
							</div>
						{/if}

						{#each canvasItems as item (item.id)}
							<CanvasItemComponent
								bind:item={canvasItems[canvasItems.findIndex((i) => i.id === item.id)]}
								selected={selectedItemId === item.id}
								{canvasRef}
								onSelect={() => selectItem(item.id)}
								onDragStart={() => {
									isDragging = true;
								}}
								onDrag={({ offsetX, offsetY }) => {
									const idx = canvasItems.findIndex((i) => i.id === item.id);
									if (idx !== -1) {
										if (snapEnabled && canvasRef) {
											const otherItemIds = canvasItems
												.filter((i) => i.id !== item.id)
												.map((i) => i.id);
											const snap = calculateSnap(
												item.id,
												offsetX,
												offsetY,
												canvasRef,
												otherItemIds
											);
											snapLines = { x: snap.snapLinesX, y: snap.snapLinesY };
											if (!item.lockX) canvasItems[idx].x = snap.x;
											if (!item.lockY) canvasItems[idx].y = snap.y;
										} else {
											snapLines = { x: [], y: [] };
											if (!item.lockX) canvasItems[idx].x = offsetX;
											if (!item.lockY) canvasItems[idx].y = offsetY;
										}
									}
								}}
								onDragEnd={() => {
									isDragging = false;
									snapLines = { x: [], y: [] };
									if (onItemMoved) onItemMoved();
								}}
							/>
						{/each}

						{#if isDragging}
							{#each snapLines.x as lineX (lineX)}
								<div
									class="snap-line-vertical pointer-events-none absolute top-0 bottom-0 w-px"
									style="left: {lineX}px; background-color: var(--input-focus);"
								></div>
							{/each}
							{#each snapLines.y as lineY (lineY)}
								<div
									class="snap-line-horizontal pointer-events-none absolute right-0 left-0 h-px"
									style="top: {lineY}px; background-color: var(--input-focus);"
								></div>
							{/each}
						{/if}
					</div>
					<!-- Canvas Height Resizer -->
					<div
						class="mx-auto h-3 w-32 cursor-row-resize rounded-b border-2 border-t-0 transition-colors hover:bg-gray-300 active:bg-gray-400"
						style="border-color: var(--border-primary); background-color: var(--bg-primary);"
						onmousedown={handleCanvasResizeStart}
						role="separator"
						aria-label="Resize canvas height"
						aria-orientation="horizontal"
					>
						<div class="flex h-full items-center justify-center">
							<div class="h-0.5 w-8 rounded" style="background-color: var(--border-primary);"></div>
						</div>
					</div>
				</div>
			</div>
		{/if}
	</div>
</div>

<style>
	.snap-line-vertical,
	.snap-line-horizontal {
		z-index: 1000;
		opacity: 0.8;
	}

	.btn-delete {
		background-color: var(--delete-button);
		transition: background-color 0.15s ease;
	}
	.btn-delete:hover {
		background-color: var(--delete-button-hover);
	}
	.btn-delete:active {
		background-color: var(--delete-button-active);
	}

	.btn-save {
		background-color: var(--create-button);
		transition: background-color 0.15s ease;
	}
	.btn-save:hover:not(:disabled) {
		background-color: var(--create-button-hover);
	}
	.btn-save:active:not(:disabled) {
		background-color: var(--create-button-active);
	}

	.btn-snap-on {
		background-color: var(--snap-on);
		color: var(--button-text);
		transition: background-color 0.15s ease;
	}
	.btn-snap-on:hover {
		background-color: var(--snap-on-hover);
	}
	.btn-snap-on:active {
		background-color: var(--snap-on-active);
	}

	.btn-snap-off {
		background-color: var(--snap-off);
		color: var(--button-text);
		transition: background-color 0.15s ease;
	}
	.btn-snap-off:hover {
		background-color: var(--snap-off-hover);
	}
	.btn-snap-off:active {
		background-color: var(--snap-off-active);
	}

	.btn-history {
		background-color: var(--history-button);
		transition: background-color 0.15s ease;
	}
	.btn-history:hover {
		background-color: var(--history-button-hover);
	}
	.btn-history:active {
		background-color: var(--history-button-active);
	}

	.tooltip {
		position: relative;
		display: inline-block;
		cursor: help;
	}

	.tooltip-text {
		visibility: hidden;
		background-color: var(--bg-secondary);
		color: var(--bg-secondary);
		font-size: 1vw;
		text-align: center;
		position: absolute;
		padding: 40%;
		border-radius: 8%;
		z-index: 1;
	}

	.tooltip:hover .tooltip-text {
		visibility: visible;
	}

	.tooltip {
		position: relative;
		display: inline-block;
		cursor: help;
	}

	.tooltip-text {
		visibility: hidden;
		background-color: var(--text-primary);
		font-size: 1vw;
		text-align: center;
		position: absolute;
		padding: 40%;
		border-radius: 8%;
		z-index: 1;
	}

	.tooltip:hover .tooltip-text {
		visibility: visible;
	}
</style>
