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
		branchId?: string | null;
		branches?: any[];
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
</script>

<div class="flex-1 overflow-auto p-6">
	<div class="mx-auto max-w-4xl">
		<div class="mb-4 flex items-center justify-between">
			<h2 class="text-3xl font-bold text-text-secondary">Canvas</h2>
			<div class="flex gap-2">
				<button
					class="rounded px-4 py-2 font-medium"
					class:btn-snap-on={snapEnabled}
					class:btn-snap-off={!snapEnabled}
					onclick={() => (snapEnabled = !snapEnabled)}
				>
					{snapEnabled ? '🧲 Snap On' : '🧲 Snap Off'}
				</button>
				{#if isEditing && onShowHistory}
					<button
						class="btn-history rounded px-4 py-2 font-medium text-white"
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
						class="btn-delete rounded px-4 py-2 font-medium text-white"
						onclick={onDeleteSelected}
					>
						Delete Selected
					</button>
				{/if}
				{#if isEditing}
					<button
						class="btn-delete rounded px-4 py-2 font-medium text-white disabled:opacity-50"
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
					{:else}
						<div class="mb-4 grow">
							<label for="branch-select" class="sr-only">Branch visibility</label>
							<select
								id="branch-select"
								bind:value={branchId}
								class="h-full w-full border-2 border-l-0 border-border-primary bg-bg-primary px-4 py-2 text-sm text-text-primary"
							>
								{#if canManageCompany}
									<option value={null}>Company Wide Visibility</option>
								{/if}
								{#each branches as branch}
									<option value={branch.id}>{branch.name}</option>
								{/each}
							</select>
						</div>
					{/if}
				</div>

				<!-- svelte-ignore a11y_no_noninteractive_element_interactions -->
				<!-- svelte-ignore a11y_no_noninteractive_tabindex -->
				<div
					bind:this={canvasRef}
					data-canvas
					class="relative min-h-125 rounded border-2 border-dashed border-border-secondary bg-bg-secondary"
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
							{snapEnabled}
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
										const snap = calculateSnap(item.id, offsetX, offsetY, canvasRef, otherItemIds);
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
		background-color: #2c7c2c;
		color: white;
		transition: background-color 0.15s ease;
	}
	.btn-snap-on:hover {
		background-color: #236b23;
	}
	.btn-snap-on:active {
		background-color: #1a5a1a;
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

	.btn-history {
		background-color: #607d8b;
		transition: background-color 0.15s ease;
	}
	.btn-history:hover {
		background-color: #546e7a;
	}
	.btn-history:active {
		background-color: #455a64;
	}
</style>
