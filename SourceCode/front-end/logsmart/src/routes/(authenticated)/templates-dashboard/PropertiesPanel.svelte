<script lang="ts">
	import type { CanvasItem } from './types';

	let {
		selectedItem,
		onUpdateProp
	}: {
		selectedItem: CanvasItem | undefined;
		onUpdateProp: (itemId: string, propKey: string, value: any) => void;
	} = $props();
</script>

{#if selectedItem}
	<div class="h-full overflow-auto p-6">
		<h3 class="mb-4 text-xl font-bold" style="color: var(--text-primary);">Properties</h3>

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
