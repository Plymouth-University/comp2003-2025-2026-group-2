<script lang="ts">
	import type { ComponentType } from './types';
	import TemperaturePicker from '$lib/components/temperature_picker.svelte';
	import UserCheckbox from '$lib/components/user_checkbox.svelte';
	import UserDropdown from '$lib/components/user_dropdown.svelte';
	import UserTextInput from '$lib/components/user_text_input.svelte';
	import UserTextLabel from '$lib/components/user_text_label.svelte';

	let {
		componentTypes,
		onDragStart
	}: { componentTypes: ComponentType[]; onDragStart: (e: DragEvent, type: string) => void } =
		$props();

	let dragPreviewRefs: Record<string, HTMLDivElement> = {};

	function handleDragStart(e: DragEvent, type: string) {
		onDragStart(e, type);
		const preview = dragPreviewRefs[type];
		if (preview && e.dataTransfer) {
			e.dataTransfer.setDragImage(preview, 0, 0);
		}
	}
</script>

<div class="drag-previews">
	<div bind:this={dragPreviewRefs['text_input']} class="drag-preview">
		<UserTextInput text="" size="medium" weight="normal" placeholder="Text Input" />
	</div>
	<div bind:this={dragPreviewRefs['checkbox']} class="drag-preview">
		<UserCheckbox text="Checkbox Label" size="16px" weight="normal" />
	</div>
	<div bind:this={dragPreviewRefs['temperature']} class="drag-preview">
		<TemperaturePicker value={0} min={-20} max={50} label="Temperature" unit="°C" />
	</div>
	<div bind:this={dragPreviewRefs['dropdown']} class="drag-preview">
		<UserDropdown selected="" options={['Option 1', 'Option 2', 'Option 3']} />
	</div>
	<div bind:this={dragPreviewRefs['label']} class="drag-preview">
		<UserTextLabel editable={false} text="Label Text" size="medium" weight="normal" />
	</div>
</div>

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

<style>
	.drag-previews {
		position: fixed;
		top: -9999px;
		left: -9999px;
		pointer-events: none;
	}

	.drag-preview {
		display: inline-block;
		width: fit-content;
		background: white;
		padding: 8px;
		border: 2px solid var(--border-primary);
		border-radius: 4px;
	}
</style>
