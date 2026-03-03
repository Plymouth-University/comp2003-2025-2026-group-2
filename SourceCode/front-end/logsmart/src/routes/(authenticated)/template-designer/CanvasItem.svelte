<script lang="ts">
	import { draggable } from '@neodrag/svelte';
	import type { CanvasItem } from './types';
	import TemperaturePicker from '$lib/components/temperature_picker.svelte';
	import UserCheckbox from '$lib/components/user_checkbox.svelte';
	import UserDropdown from '$lib/components/user_dropdown.svelte';
	import UserTextInput from '$lib/components/user_text_input.svelte';
	import UserTextLabel from '$lib/components/user_text_label.svelte';

	let {
		item = $bindable(),
		selected,
		canvasRef,
		onDragStart,
		onDrag,
		onDragEnd,
		onSelect
	} = $props<{
		item: CanvasItem;
		selected: boolean;
		canvasRef: HTMLElement | null;
		onDragStart: () => void;
		onDrag: (e: { offsetX: number; offsetY: number }) => void;
		onDragEnd: () => void;
		onSelect: () => void;
	}>();
</script>

<div
	data-item-id={item.id}
	class="canvas-item absolute cursor-move rounded p-2"
	class:border-2={selected}
	class:ring-2={selected}
	class:ring-blue-500={selected}
	class:selected-item={selected}
	class:bg-bg-primary={true}
	style="left: {item.x}px; top: {item.y}px; transform: none !important;"
	use:draggable={{
		position: { x: item.x, y: item.y },
		bounds: canvasRef,
		axis: item.lockX && item.lockY ? undefined : item.lockX ? 'y' : item.lockY ? 'x' : 'both',
		disabled: item.lockX && item.lockY,
		transform: () => '',
		onDragStart,
		onDrag,
		onDragEnd
	}}
	onclick={(e) => {
		e.stopPropagation();
		onSelect();
	}}
	onkeydown={(e) => {
		if (e.key === 'Enter') onSelect();
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
		<UserCheckbox text={item.props.text} size={item.props.size} weight={item.props.weight} />
	{:else if item.type === 'temperature'}
		<TemperaturePicker
			bind:value={item.props.value}
			min={item.props.min}
			max={item.props.max}
			label={item.props.label}
			unit={item.props.unit}
		/>
	{:else if item.type === 'dropdown'}
		<UserDropdown
			bind:selected={item.props.selected}
			options={item.props.options}
			disabled={true}
		/>
	{:else if item.type === 'label'}
		<UserTextLabel
			editable={item.props.editable}
			bind:text={item.props.text}
			size={item.props.size}
			weight={item.props.weight}
		/>
	{/if}
</div>

<style>
	.canvas-item {
		touch-action: none;
	}
	.selected-item {
		border-color: var(--border-primary);
	}
</style>
