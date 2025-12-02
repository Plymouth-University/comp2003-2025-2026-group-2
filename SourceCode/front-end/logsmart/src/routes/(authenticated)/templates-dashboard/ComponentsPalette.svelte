<script lang="ts">
	import TemperaturePicker from '$lib/components/temperature_picker.svelte';
	import UserCheckbox from '$lib/components/user_checkbox.svelte';
	import UserDropdown from '$lib/components/user_dropdown.svelte';
	import UserTextInput from '$lib/components/user_text_input.svelte';
	import UserTextLabel from '$lib/components/user_text_label.svelte';
	import type { ComponentType } from './types';

	let {
		componentTypes,
		onAddComponent
	}: {
		componentTypes: ComponentType[];
		onAddComponent: (type: string, x: number, y: number) => void;
	} = $props();

	let draggingType = $state<string | null>(null);
	let ghostPosition = $state({ x: 0, y: 0 });
	let ghostRef = $state<HTMLDivElement | null>(null);

	function handleMouseDown(e: MouseEvent, type: string) {
		e.preventDefault();
		draggingType = type;
		ghostPosition = { x: e.clientX, y: e.clientY };

		window.addEventListener('mousemove', handleMouseMove);
		window.addEventListener('mouseup', handleMouseUp);
	}

	function handleMouseMove(e: MouseEvent) {
		if (!draggingType) return;
		ghostPosition = { x: e.clientX, y: e.clientY };
	}

	const pastelColors: Record<string, string> = {
		text_input: '#e0f2fe', // light blue
		checkbox: '#dcfce7', // light green
		temperature: '#fef3c7', // light amber
		dropdown: '#f3e8ff', // light purple
		label: '#ffe4e6' // light rose
	};

	function handleMouseUp(e: MouseEvent) {
		window.removeEventListener('mousemove', handleMouseMove);
		window.removeEventListener('mouseup', handleMouseUp);

		if (!draggingType) return;

		const canvas = document.querySelector('[data-canvas]');
		if (canvas && ghostRef) {
			const canvasRect = canvas.getBoundingClientRect();
			const ghostRect = ghostRef.getBoundingClientRect();

			if (
				e.clientX >= canvasRect.left &&
				e.clientX <= canvasRect.right &&
				e.clientY >= canvasRect.top &&
				e.clientY <= canvasRect.bottom
			) {
				const dropX = e.clientX - canvasRect.left - ghostRect.width / 2;
				const dropY = e.clientY - canvasRect.top - ghostRect.height / 2;
				onAddComponent(draggingType, Math.max(0, dropX), Math.max(0, dropY));
			}
		}

		draggingType = null;
	}
</script>

<div class="h-full p-4">
	<h2 class="mb-3 text-xl font-bold" style="color: var(--text-primary);">Components</h2>

	<div class="space-y-1">
		{#each componentTypes as component (component.type)}
			<div
				class="flex cursor-grab items-center gap-2 rounded border px-3 py-2 text-sm hover:opacity-80 active:cursor-grabbing"
				style="border-color: var(--border-primary); background-color: {pastelColors[component.type] || 'transparent'};"
				onmousedown={(e) => handleMouseDown(e, component.type)}
				role="button"
				tabindex="0"
			>
				<div
					class="flex h-6 w-6 items-center justify-center border text-xs font-bold"
					style="border-color: var(--border-primary); color: var(--text-primary);"
				>
					{component.icon}
				</div>
				<span style="color: var(--text-primary);">{component.name}</span>
			</div>
		{/each}
	</div>
</div>

{#if draggingType}
	<div
		bind:this={ghostRef}
		class="pointer-events-none fixed z-50 -translate-x-1/2 -translate-y-1/2 rounded border-2 bg-white p-2 shadow-lg"
		style="left: {ghostPosition.x}px; top: {ghostPosition.y}px; border-color: var(--border-primary);"
	>
		{#if draggingType === 'text_input'}
			<UserTextInput text="" size={16} weight="normal" placeholder="Text Input" />
		{:else if draggingType === 'checkbox'}
			<UserCheckbox text="Checkbox Label" size="16px" weight="normal" />
		{:else if draggingType === 'temperature'}
			<TemperaturePicker value={0} min={-20} max={50} label="Temperature" unit="°C" />
		{:else if draggingType === 'dropdown'}
			<UserDropdown selected="" options={['Option 1', 'Option 2', 'Option 3']} />
		{:else if draggingType === 'label'}
			<UserTextLabel editable={false} text="Label Text" size={16} weight="normal" />
		{/if}
	</div>
{/if}
