<script lang="ts">
	import type { CanvasItem } from './types';
	import { isValidCSSColor } from '$lib/utils/validation';

	let {
		selectedItem,
		onUpdateProp,
		onAlign
	}: {
		selectedItem: CanvasItem | undefined;
		onUpdateProp: (itemId: string, propKey: string, value: unknown) => void;
		onAlign: (
			itemId: string,
			horizontal: 'left' | 'center' | 'right' | null,
			vertical: 'top' | 'center' | 'bottom' | null
		) => void;
	} = $props();
</script>

{#if selectedItem}
	<div class="h-full overflow-auto p-6">
		<h3 class="mb-4 text-xl font-bold text-text-primary">Properties</h3>

		<!-- Alignment Options -->
		<div class="mb-4 space-y-3 border-b-2 border-border-primary pb-4">
			<span class="mb-2 block text-sm font-medium text-text-secondary"
				>Alignment</span
			>
			<div class="grid grid-cols-3 gap-1">
				<div></div>
				<button
					class="alignment-button flex h-10 flex-1 items-center justify-center gap-1 rounded border-2 border-border-primary text-xs text-text-primary transition-colors"
					class:opacity-50={selectedItem.lockY}
					class:cursor-not-allowed={selectedItem.lockY}
					onclick={() => onAlign(selectedItem.id, null, 'top')}
					disabled={selectedItem.lockY}
					title={selectedItem.lockY ? 'Disabled - Vertical lock is enabled' : 'Align Top'}
				>
					<span>⬆</span><span>Top</span>
				</button>
				<div></div>

				<button
					class="alignment-button flex h-10 flex-1 items-center justify-center gap-1 rounded border-2 border-border-primary text-text-primary text-xs transition-colors"
					class:opacity-50={selectedItem.lockX}
					class:cursor-not-allowed={selectedItem.lockX}
					onclick={() => onAlign(selectedItem.id, 'left', null)}
					disabled={selectedItem.lockX}
					title={selectedItem.lockX ? 'Disabled - Horizontal lock is enabled' : 'Align Left'}
				>
					<span>⬅</span><span>Left</span>
				</button>
				<button
					class="alignment-button flex h-10 flex-1 items-center justify-center gap-1 rounded border-2 border-border-primary text-text-primary text-xs transition-colors"
					class:opacity-50={selectedItem.lockX && selectedItem.lockY}
					class:cursor-not-allowed={selectedItem.lockX && selectedItem.lockY}
					onclick={() => onAlign(selectedItem.id, 'center', 'center')}
					disabled={selectedItem.lockX && selectedItem.lockY}
					title={selectedItem.lockX && selectedItem.lockY
						? 'Disabled - Both locks enabled'
						: 'Center Both'}
				>
					<span>⊕</span><span>Center</span>
				</button>
				<button
					class="alignment-button flex h-10 flex-1 items-center justify-center gap-1 rounded border-2 border-border-primary text-text-primary text-xs transition-colors"
					class:opacity-50={selectedItem.lockX}
					class:cursor-not-allowed={selectedItem.lockX}
					onclick={() => onAlign(selectedItem.id, 'right', null)}
					disabled={selectedItem.lockX}
					title={selectedItem.lockX ? 'Disabled - Horizontal lock is enabled' : 'Align Right'}
				>
					<span>Right</span><span>➡</span>
				</button>

				<div></div>
				<button
					class="alignment-button flex h-10 flex-1 items-center justify-center gap-1 rounded border-2 border-border-primary text-text-primary text-xs transition-colors"
					class:opacity-50={selectedItem.lockY}
					class:cursor-not-allowed={selectedItem.lockY}
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
		<div class="mb-4 space-y-2 border-b-2 border-border-primary pb-4">
			<span class="mb-2 block text-sm font-medium text-text-secondary"
				>Position</span
			>
			<div class="flex items-center gap-2">
				<span class="w-3 text-sm text-text-primary">X</span>
				<input
					type="number"
					value={Math.round(selectedItem.x)}
					oninput={(e) => onUpdateProp(selectedItem.id, 'x', parseInt(e.currentTarget.value) || 0)}
					class="w-20 border-2 border-border-primary bg-bg-primary px-2 py-1 text-sm text-text-primary"
					class:opacity-50={selectedItem.lockX}
					disabled={selectedItem.lockX}
				/>
				<button
					class="lock-button flex h-8 w-8 items-center justify-center rounded border-2 border-border-primary text-sm text-text-primary transition-colors"
					class:bg-bg-secondary={selectedItem.lockX}
					onclick={() => onUpdateProp(selectedItem.id, 'lockX', !selectedItem.lockX)}
					title={selectedItem.lockX ? 'Unlock X position' : 'Lock X position'}
				>
					{selectedItem.lockX ? '🔒' : '🔓'}
				</button>
			</div>
			<div class="flex items-center gap-2">
				<span class="w-3 text-sm text-text-primary">Y</span>
				<input
					type="number"
					value={Math.round(selectedItem.y)}
					oninput={(e) => onUpdateProp(selectedItem.id, 'y', parseInt(e.currentTarget.value) || 0)}
					class="w-20 border-2 border-border-primary bg-bg-primary px-2 py-1 text-sm text-text-primary"
					class:opacity-50={selectedItem.lockY}
					disabled={selectedItem.lockY}
				/>
				<button
					class="lock-button flex h-8 w-8 items-center justify-center rounded border-2 border-border-primary text-sm text-text-primary transition-colors"
					class:bg-bg-secondary={selectedItem.lockY}
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
					<label class="mb-1 block text-sm font-medium text-text-secondary">
						Placeholder
						<input
							type="text"
							value={selectedItem.props.placeholder}
							oninput={(e) => onUpdateProp(selectedItem.id, 'placeholder', e.currentTarget.value)}
							class="mt-1 w-full border-2 px-3 py-2 border-border-primary bg-bg-primary text-text-primary"
						/>
					</label>
				</div>
				<div>
					<label class="mb-1 block text-sm font-medium text-text-secondary">
						Size (px)
						<select
							value={selectedItem.props.size}
							onchange={(e) =>
								onUpdateProp(selectedItem.id, 'size', parseInt(e.currentTarget.value))}
							class="mt-1 w-full border-2 px-3 py-2 border-border-primary bg-bg-primary text-text-primary"
						>
							{#each [10, 12, 14, 16, 18, 20, 24, 28, 32, 36, 40, 48] as s (s)}
								<option value={s}>{s}px</option>
							{/each}
						</select>
					</label>
				</div>
				<div>
					<label class="mb-1 block text-sm font-medium text-text-secondary">
						Weight
						<select
							value={selectedItem.props.weight}
							onchange={(e) => onUpdateProp(selectedItem.id, 'weight', e.currentTarget.value)}
							class="mt-1 w-full border-2 px-3 py-2 border-border-primary bg-bg-primary text-text-primary"
						>
							<option value="light">Light</option>
							<option value="normal">Normal</option>
							<option value="bold">Bold</option>
						</select>
					</label>
				</div>
				<div>
					<label class="mb-1 block text-sm font-medium text-text-secondary">
						Font Family
						<select
							value={selectedItem.props.fontFamily}
							onchange={(e) => onUpdateProp(selectedItem.id, 'fontFamily', e.currentTarget.value)}
							class="mt-1 w-full border-2 px-3 py-2 border-border-primary bg-bg-primary text-text-primary"
						>
							<option value="system-ui">System UI</option>
							<option value="serif">Serif</option>
							<option value="sans-serif">Sans-Serif</option>
							<option value="monospace">Monospace</option>
						</select>
					</label>
				</div>
				<div>
					<label class="mb-1 block text-sm font-medium text-text-secondary">
						Text Decoration
						<select
							value={selectedItem.props.textDecoration}
							onchange={(e) =>
								onUpdateProp(selectedItem.id, 'textDecoration', e.currentTarget.value)}
							class="mt-1 w-full border-2 px-3 py-2 border-border-primary bg-bg-primary text-text-primary"
						>
							<option value="none">None</option>
							<option value="underline">Underline</option>
							<option value="line-through">Line Through</option>
						</select>
					</label>
				</div>
				<div>
					<label class="mb-1 block text-sm font-medium text-text-secondary">
						Color
						<div class="mt-1 flex items-center gap-2">
							<input
								type="color"
								value={selectedItem.props.color || '#000000'}
								oninput={(e) => {
									const newColor = e.currentTarget.value;
									if (isValidCSSColor(newColor)) {
										onUpdateProp(selectedItem.id, 'color', newColor);
									}
								}}
								class="h-12 w-12 min-w-12 cursor-pointer rounded border-2 p-1 border-border-primary"
							/>
							<input
								type="text"
								value={selectedItem.props.color || ''}
								oninput={(e) => {
									const newColor = e.currentTarget.value;
									if (isValidCSSColor(newColor)) {
										onUpdateProp(selectedItem.id, 'color', newColor);
									}
								}}
								placeholder="#000000"
								class="w-2 flex-1 border-2 px-3 py-2 border-border-primary bg-bg-primary text-text-primary"
							/>
						</div>
					</label>
				</div>
				<div>
					<label class="mb-1 block text-sm font-medium text-text-secondary">
						Font Family
						<select
							value={selectedItem.props.fontFamily}
							onchange={(e) => onUpdateProp(selectedItem.id, 'fontFamily', e.currentTarget.value)}
							class="mt-1 w-full border-2 px-3 py-2 border-border-primary bg-bg-primary text-text-primary"
						>
							<option value="text">Text</option>
							<option value="int">Integer</option>
							<option value="float">Float</option>
						</select>
					</label>
				</div>
				<div>
					<label class="mb-1 block text-sm font-medium text-text-secondary">
						Max Length
						<input
							type="number"
							value={selectedItem.props.maxLength !== null &&
							selectedItem.props.maxLength !== undefined
								? selectedItem.props.maxLength
								: ''}
							oninput={(e) => {
								const value = e.currentTarget.value ? parseInt(e.currentTarget.value) : null;
								// Only update if value is null or >= 0
								if (value === null || value >= 0) {
									onUpdateProp(selectedItem.id, 'maxLength', value);
								}
							}}
							placeholder="No limit"
							class="mt-1 w-full border-2 px-3 py-2 border-border-primary bg-bg-primary text-text-primary"
						/>
					</label>
				</div>
				<div>
					<label class="mb-1 block text-sm font-medium text-text-secondary">
						Min Length
						<input
							type="number"
							value={selectedItem.props.minLength !== null &&
							selectedItem.props.minLength !== undefined
								? selectedItem.props.minLength
								: ''}
							oninput={(e) => {
								const value = e.currentTarget.value ? parseInt(e.currentTarget.value) : null;
								// Only update if value is null or >= 0
								if (value === null || value >= 0) {
									onUpdateProp(selectedItem.id, 'minLength', value);
								}
							}}
							placeholder="No limit"
							class="mt-1 w-full border-2 px-3 py-2 border-border-primary bg-bg-primary text-text-primary"
						/>
					</label>
				</div>
				<div class="flex items-center gap-2">
					<input
						type="checkbox"
						id="required"
						checked={!!selectedItem.props.required}
						onchange={(e) => onUpdateProp(selectedItem.id, 'required', e.currentTarget.checked)}
						class="h-4 w-4"
					/>
					<label for="required" class="text-sm font-medium text-text-secondary"
						>Required</label
					>
				</div>
			</div>
		{:else if selectedItem.type === 'label'}
			<div class="space-y-2">
				<div>
					<span class="mb-1 block text-sm font-medium text-text-secondary"
						>Text</span
					>
					<input
						type="text"
						value={selectedItem.props.text}
						oninput={(e) => onUpdateProp(selectedItem.id, 'text', e.currentTarget.value)}
						class="w-full border-2 px-3 py-2"
						class="border-border-primary bg-bg-primary text-text-primary"
					/>
				</div>
				<div>
					<span class="mb-1 block text-sm font-medium text-text-secondary"
						>Size (px)</span
					>
					<select
						value={selectedItem.props.size}
						onchange={(e) => onUpdateProp(selectedItem.id, 'size', parseInt(e.currentTarget.value))}
						class="w-full border-2 px-3 py-2"
						class="border-border-primary bg-bg-primary text-text-primary"
					>
						{#each [10, 12, 14, 16, 18, 20, 24, 28, 32, 36, 40, 48] as s (s)}
							<option value={s}>{s}px</option>
						{/each}
					</select>
				</div>
				<div>
					<span class="mb-1 block text-sm font-medium text-text-secondary"
						>Weight</span
					>
					<select
						value={selectedItem.props.weight}
						onchange={(e) => onUpdateProp(selectedItem.id, 'weight', e.currentTarget.value)}
						class="w-full border-2 px-3 py-2"
						class="border-border-primary bg-bg-primary text-text-primary"
					>
						<option value="light">Light</option>
						<option value="normal">Normal</option>
						<option value="bold">Bold</option>
					</select>
				</div>
				<div>
					<label class="mb-1 block text-sm font-medium text-text-secondary">
						Font Family
						<select
							value={selectedItem.props.fontFamily}
							onchange={(e) => onUpdateProp(selectedItem.id, 'fontFamily', e.currentTarget.value)}
							class="mt-1 w-full border-2 px-3 py-2"
							class="border-border-primary bg-bg-primary text-text-primary"
						>
							<option value="system-ui">System UI</option>
							<option value="serif">Serif</option>
							<option value="sans-serif">Sans-Serif</option>
							<option value="monospace">Monospace</option>
						</select>
					</label>
				</div>
				<div>
					<label class="mb-1 block text-sm font-medium text-text-secondary">
						Text Decoration
						<select
							value={selectedItem.props.textDecoration}
							onchange={(e) =>
								onUpdateProp(selectedItem.id, 'textDecoration', e.currentTarget.value)}
							class="mt-1 w-full border-2 px-3 py-2"
							class="border-border-primary bg-bg-primary text-text-primary"
						>
							<option value="none">None</option>
							<option value="underline">Underline</option>
							<option value="line-through">Line Through</option>
						</select>
					</label>
				</div>
				<div>
					<label class="mb-1 block text-sm font-medium text-text-secondary">
						Color
						<div class="mt-1 flex items-center gap-2">
							<input
								type="color"
								value={selectedItem.props.color || '#000000'}
								oninput={(e) => onUpdateProp(selectedItem.id, 'color', e.currentTarget.value)}
								class="h-12 w-12 min-w-12 cursor-pointer rounded border-2 p-1"
								class="border-border-primary"
							/>
							<input
								type="text"
								value={selectedItem.props.color || ''}
								oninput={(e) => onUpdateProp(selectedItem.id, 'color', e.currentTarget.value)}
								placeholder="#000000"
								class="w-2 flex-1 border-2 px-3 py-2"
								class="border-border-primary bg-bg-primary text-text-primary"
							/>
						</div>
					</label>
				</div>
			</div>
		{:else if selectedItem.type === 'checkbox'}
			<div class="space-y-2">
				<div>
					<span class="mb-1 block text-sm font-medium text-text-secondary"
						>Label</span
					>
					<input
						type="text"
						value={selectedItem.props.text}
						oninput={(e) => onUpdateProp(selectedItem.id, 'text', e.currentTarget.value)}
						class="w-full border-2 px-3 py-2"
						class="border-border-primary bg-bg-primary text-text-primary"
					/>
				</div>
				<div>
					<label class="mb-1 block text-sm font-medium text-text-secondary">
						Color
						<div class="mt-1 flex items-center gap-2">
							<input
								type="color"
								value={selectedItem.props.color || '#000000'}
								oninput={(e) => onUpdateProp(selectedItem.id, 'color', e.currentTarget.value)}
								class="h-12 w-12 min-w-12 cursor-pointer rounded border-2 p-1"
								class="border-border-primary"
							/>
							<input
								type="text"
								value={selectedItem.props.color || ''}
								oninput={(e) => onUpdateProp(selectedItem.id, 'color', e.currentTarget.value)}
								placeholder="#000000"
								class="w-2 flex-1 border-2 px-3 py-2"
								class="border-border-primary bg-bg-primary text-text-primary"
							/>
						</div>
					</label>
				</div>
				<div class="flex items-center gap-2">
					<input
						type="checkbox"
						id="checkbox-required"
						checked={!!selectedItem.props.required}
						onchange={(e) => onUpdateProp(selectedItem.id, 'required', e.currentTarget.checked)}
						class="h-4 w-4"
					/>
					<label
						for="checkbox-required"
						class="text-sm font-medium"
						class="text-text-secondary">Required</label
					>
				</div>
			</div>
		{:else if selectedItem.type === 'temperature'}
			<div class="space-y-2">
				<div>
					<span class="mb-1 block text-sm font-medium text-text-secondary"
						>Label</span
					>
					<input
						type="text"
						value={selectedItem.props.label}
						oninput={(e) => onUpdateProp(selectedItem.id, 'label', e.currentTarget.value)}
						class="w-full border-2 px-3 py-2"
						class="border-border-primary bg-bg-primary text-text-primary"
					/>
				</div>
				<div>
					<span class="mb-1 block text-sm font-medium text-text-secondary"
						>Unit</span
					>
					<select
						value={selectedItem.props.unit}
						onchange={(e) => onUpdateProp(selectedItem.id, 'unit', e.currentTarget.value)}
						class="w-full border-2 px-3 py-2"
						class="border-border-primary bg-bg-primary text-text-primary"
					>
						<option value="°C">°C (Celsius)</option>
						<option value="°F">°F (Fahrenheit)</option>
					</select>
				</div>
				<div>
					<span class="mb-1 block text-sm font-medium text-text-secondary"
						>Min</span
					>
					<input
						type="number"
						value={selectedItem.props.min}
						oninput={(e) => onUpdateProp(selectedItem.id, 'min', parseInt(e.currentTarget.value))}
						class="w-full border-2 px-3 py-2"
						class="border-border-primary bg-bg-primary text-text-primary"
					/>
				</div>
				<div>
					<span class="mb-1 block text-sm font-medium text-text-secondary"
						>Max</span
					>
					<input
						type="number"
						value={selectedItem.props.max}
						oninput={(e) => onUpdateProp(selectedItem.id, 'max', parseInt(e.currentTarget.value))}
						class="w-full border-2 px-3 py-2"
						class="border-border-primary bg-bg-primary text-text-primary"
					/>
				</div>
			</div>
		{:else if selectedItem.type === 'dropdown'}
			<div class="space-y-2">
				<div>
					<span class="mb-1 block text-sm font-medium text-text-secondary"
						>Options (one per line)</span
					>
					<textarea
						value={Array.isArray(selectedItem.props.options)
							? selectedItem.props.options.join('\n')
							: ''}
						oninput={(e) =>
							onUpdateProp(
								selectedItem.id,
								'options',
								e.currentTarget.value.split('\n').filter((o: string) => o.trim())
							)}
						class="h-32 w-full border-2 px-3 py-2"
						class="border-border-primary bg-bg-primary text-text-primary"
					></textarea>
				</div>
			</div>
		{/if}
	</div>
{/if}

<style>
	.alignment-button {
		background-color: var(--bg-primary);
	}

	.alignment-button:hover:not(:disabled) {
		background-color: var(--bg-secondary);
	}

	.lock-button {
		background-color: var(--bg-primary);
	}

	.lock-button:hover:not(:disabled) {
		background-color: var(--bg-secondary);
	}
</style>
