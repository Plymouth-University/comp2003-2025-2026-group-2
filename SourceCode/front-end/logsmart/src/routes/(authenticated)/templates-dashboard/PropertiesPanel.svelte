<script lang="ts">
	import type { CanvasItem } from './types';

	let {
		selectedItem,
		onUpdateProp,
		onAlign
	}: {
		selectedItem: CanvasItem | undefined;
		onUpdateProp: (itemId: string, propKey: string, value: any) => void;
		onAlign: (
			itemId: string,
			horizontal: 'left' | 'center' | 'right' | null,
			vertical: 'top' | 'center' | 'bottom' | null
		) => void;
	} = $props();
</script>

{#if selectedItem}
	<div class="h-full overflow-auto p-6">
		<h3 class="mb-4 text-xl font-bold" style="color: var(--text-primary);">Properties</h3>

		<!-- Alignment Options -->
		<div class="mb-4 space-y-3 border-b-2 pb-4" style="border-color: var(--border-primary);">
			<span class="mb-2 block text-sm font-medium" style="color: var(--text-secondary);"
				>Alignment</span
			>
			<div class="grid grid-cols-3 gap-1">
				<div></div>
				<button
					class="flex h-10 flex-1 items-center justify-center gap-1 rounded border-2 text-xs transition-colors"
					class:opacity-50={selectedItem.lockY}
					class:cursor-not-allowed={selectedItem.lockY}
					class:hover:bg-gray-100={!selectedItem.lockY}
					style="border-color: var(--border-primary); color: var(--text-primary);"
					onclick={() => onAlign(selectedItem.id, null, 'top')}
					disabled={selectedItem.lockY}
					title={selectedItem.lockY ? 'Disabled - Vertical lock is enabled' : 'Align Top'}
				>
					<span>⬆</span><span>Top</span>
				</button>
				<div></div>

				<button
					class="flex h-10 flex-1 items-center justify-center gap-1 rounded border-2 text-xs transition-colors"
					class:opacity-50={selectedItem.lockX}
					class:cursor-not-allowed={selectedItem.lockX}
					class:hover:bg-gray-100={!selectedItem.lockX}
					style="border-color: var(--border-primary); color: var(--text-primary);"
					onclick={() => onAlign(selectedItem.id, 'left', null)}
					disabled={selectedItem.lockX}
					title={selectedItem.lockX ? 'Disabled - Horizontal lock is enabled' : 'Align Left'}
				>
					<span>⬅</span><span>Left</span>
				</button>
				<button
					class="flex h-10 flex-1 items-center justify-center gap-1 rounded border-2 text-xs transition-colors"
					class:opacity-50={selectedItem.lockX && selectedItem.lockY}
					class:cursor-not-allowed={selectedItem.lockX && selectedItem.lockY}
					class:hover:bg-gray-100={!(selectedItem.lockX && selectedItem.lockY)}
					style="border-color: var(--border-primary); color: var(--text-primary);"
					onclick={() => onAlign(selectedItem.id, 'center', 'center')}
					disabled={selectedItem.lockX && selectedItem.lockY}
					title={selectedItem.lockX && selectedItem.lockY
						? 'Disabled - Both locks enabled'
						: 'Center Both'}
				>
					<span>⊕</span><span>Center</span>
				</button>
				<button
					class="flex h-10 flex-1 items-center justify-center gap-1 rounded border-2 text-xs transition-colors"
					class:opacity-50={selectedItem.lockX}
					class:cursor-not-allowed={selectedItem.lockX}
					class:hover:bg-gray-100={!selectedItem.lockX}
					style="border-color: var(--border-primary); color: var(--text-primary);"
					onclick={() => onAlign(selectedItem.id, 'right', null)}
					disabled={selectedItem.lockX}
					title={selectedItem.lockX ? 'Disabled - Horizontal lock is enabled' : 'Align Right'}
				>
					<span>Right</span><span>➡</span>
				</button>

				<div></div>
				<button
					class="flex h-10 flex-1 items-center justify-center gap-1 rounded border-2 text-xs transition-colors"
					class:opacity-50={selectedItem.lockY}
					class:cursor-not-allowed={selectedItem.lockY}
					class:hover:bg-gray-100={!selectedItem.lockY}
					style="border-color: var(--border-primary); color: var(--text-primary);"
					onclick={() => onAlign(selectedItem.id, null, 'bottom')}
					disabled={selectedItem.lockY}
					title={selectedItem.lockY ? 'Disabled - Vertical lock is enabled' : 'Align Bottom'}
				>
					<span>Bottom</span><span>⬇</span>
				</button>
				<div></div>
			</div>
		</div>

		<!-- Position -->
		<div class="mb-4 space-y-2 border-b-2 pb-4" style="border-color: var(--border-primary);">
			<span class="mb-2 block text-sm font-medium" style="color: var(--text-secondary);"
				>Position</span
			>
			<div class="flex items-center gap-2">
				<span class="w-3 text-sm" style="color: var(--text-primary);">X</span>
				<input
					type="number"
					value={Math.round(selectedItem.x)}
					oninput={(e) => onUpdateProp(selectedItem.id, 'x', parseInt(e.currentTarget.value) || 0)}
					class="w-20 border-2 px-2 py-1 text-sm"
					class:opacity-50={selectedItem.lockX}
					style="border-color: var(--border-primary); color: var(--text-primary);"
					disabled={selectedItem.lockX}
				/>
				<button
					class="flex h-8 w-8 items-center justify-center rounded border-2 text-sm transition-colors"
					class:bg-blue-100={selectedItem.lockX}
					class:hover:bg-gray-100={!selectedItem.lockX}
					style="border-color: var(--border-primary); color: var(--text-primary);"
					onclick={() => onUpdateProp(selectedItem.id, 'lockX', !selectedItem.lockX)}
					title={selectedItem.lockX ? 'Unlock X position' : 'Lock X position'}
				>
					{selectedItem.lockX ? '🔒' : '🔓'}
				</button>
			</div>
			<div class="flex items-center gap-2">
				<span class="w-3 text-sm" style="color: var(--text-primary);">Y</span>
				<input
					type="number"
					value={Math.round(selectedItem.y)}
					oninput={(e) => onUpdateProp(selectedItem.id, 'y', parseInt(e.currentTarget.value) || 0)}
					class="w-20 border-2 px-2 py-1 text-sm"
					class:opacity-50={selectedItem.lockY}
					style="border-color: var(--border-primary); color: var(--text-primary);"
					disabled={selectedItem.lockY}
				/>
				<button
					class="flex h-8 w-8 items-center justify-center rounded border-2 text-sm transition-colors"
					class:bg-blue-100={selectedItem.lockY}
					class:hover:bg-gray-100={!selectedItem.lockY}
					style="border-color: var(--border-primary); color: var(--text-primary);"
					onclick={() => onUpdateProp(selectedItem.id, 'lockY', !selectedItem.lockY)}
					title={selectedItem.lockY ? 'Unlock Y position' : 'Lock Y position'}
				>
					{selectedItem.lockY ? '🔒' : '🔓'}
				</button>
			</div>
		</div>

		{#if selectedItem.type === 'text_input'}
			<div class="space-y-2">
				<div>
					<span class="mb-1 block text-sm font-medium" style="color: var(--text-secondary);"
						>Placeholder</span
					>
					<input
						type="text"
						value={selectedItem.props.placeholder}
						oninput={(e) => onUpdateProp(selectedItem.id, 'placeholder', e.currentTarget.value)}
						class="w-full border-2 px-3 py-2"
						style="border-color: var(--border-primary); color: var(--text-primary);"
					/>
				</div>
				<div>
					<span class="mb-1 block text-sm font-medium" style="color: var(--text-secondary);"
						>Size (px)</span
					>
					<select
						value={selectedItem.props.size}
						onchange={(e) => onUpdateProp(selectedItem.id, 'size', parseInt(e.currentTarget.value))}
						class="w-full border-2 px-3 py-2"
						style="border-color: var(--border-primary); color: var(--text-primary);"
					>
						{#each [10, 12, 14, 16, 18, 20, 24, 28, 32, 36, 40, 48] as s}
							<option value={s}>{s}px</option>
						{/each}
					</select>
				</div>
				<div>
					<span class="mb-1 block text-sm font-medium" style="color: var(--text-secondary);"
						>Weight</span
					>
					<select
						value={selectedItem.props.weight}
						onchange={(e) => onUpdateProp(selectedItem.id, 'weight', e.currentTarget.value)}
						class="w-full border-2 px-3 py-2"
						style="border-color: var(--border-primary); color: var(--text-primary);"
					>
						<option value="light">Light</option>
						<option value="normal">Normal</option>
						<option value="bold">Bold</option>
					</select>
				</div>
			</div>
		{:else if selectedItem.type === 'label'}
			<div class="space-y-2">
				<div>
					<span class="mb-1 block text-sm font-medium" style="color: var(--text-secondary);"
						>Text</span
					>
					<input
						type="text"
						value={selectedItem.props.text}
						oninput={(e) => onUpdateProp(selectedItem.id, 'text', e.currentTarget.value)}
						class="w-full border-2 px-3 py-2"
						style="border-color: var(--border-primary); color: var(--text-primary);"
					/>
				</div>
				<div>
					<span class="mb-1 block text-sm font-medium" style="color: var(--text-secondary);"
						>Size (px)</span
					>
					<select
						value={selectedItem.props.size}
						onchange={(e) => onUpdateProp(selectedItem.id, 'size', parseInt(e.currentTarget.value))}
						class="w-full border-2 px-3 py-2"
						style="border-color: var(--border-primary); color: var(--text-primary);"
					>
						{#each [10, 12, 14, 16, 18, 20, 24, 28, 32, 36, 40, 48] as s}
							<option value={s}>{s}px</option>
						{/each}
					</select>
				</div>
				<div>
					<span class="mb-1 block text-sm font-medium" style="color: var(--text-secondary);"
						>Weight</span
					>
					<select
						value={selectedItem.props.weight}
						onchange={(e) => onUpdateProp(selectedItem.id, 'weight', e.currentTarget.value)}
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
			<div class="space-y-2">
				<div>
					<span class="mb-1 block text-sm font-medium" style="color: var(--text-secondary);"
						>Label</span
					>
					<input
						type="text"
						value={selectedItem.props.text}
						oninput={(e) => onUpdateProp(selectedItem.id, 'text', e.currentTarget.value)}
						class="w-full border-2 px-3 py-2"
						style="border-color: var(--border-primary); color: var(--text-primary);"
					/>
				</div>
			</div>
		{:else if selectedItem.type === 'temperature'}
			<div class="space-y-2">
				<div>
					<span class="mb-1 block text-sm font-medium" style="color: var(--text-secondary);"
						>Label</span
					>
					<input
						type="text"
						value={selectedItem.props.label}
						oninput={(e) => onUpdateProp(selectedItem.id, 'label', e.currentTarget.value)}
						class="w-full border-2 px-3 py-2"
						style="border-color: var(--border-primary); color: var(--text-primary);"
					/>
				</div>
				<div>
					<span class="mb-1 block text-sm font-medium" style="color: var(--text-secondary);"
						>Unit</span
					>
					<select
						value={selectedItem.props.unit}
						onchange={(e) => onUpdateProp(selectedItem.id, 'unit', e.currentTarget.value)}
						class="w-full border-2 px-3 py-2"
						style="border-color: var(--border-primary); color: var(--text-primary);"
					>
						<option value="°C">°C (Celsius)</option>
						<option value="°F">°F (Fahrenheit)</option>
					</select>
				</div>
				<div>
					<span class="mb-1 block text-sm font-medium" style="color: var(--text-secondary);"
						>Min</span
					>
					<input
						type="number"
						value={selectedItem.props.min}
						oninput={(e) => onUpdateProp(selectedItem.id, 'min', parseInt(e.currentTarget.value))}
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
						oninput={(e) => onUpdateProp(selectedItem.id, 'max', parseInt(e.currentTarget.value))}
						class="w-full border-2 px-3 py-2"
						style="border-color: var(--border-primary); color: var(--text-primary);"
					/>
				</div>
			</div>
		{:else if selectedItem.type === 'dropdown'}
			<div class="space-y-2">
				<div>
					<span class="mb-1 block text-sm font-medium" style="color: var(--text-secondary);"
						>Options (one per line)</span
					>
					<textarea
						value={selectedItem.props.options.join('\n')}
						oninput={(e) =>
							onUpdateProp(
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
