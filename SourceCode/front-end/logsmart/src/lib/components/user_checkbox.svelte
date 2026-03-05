<script lang="ts">
	import { sanitizeColorValue } from '$lib/utils/validation';

	let {
		text = 'Checkbox Label',
		size = '16px',
		weight = 'normal',
		checked = $bindable(false),
		disabled = false,
		color = '',
		required = false
	}: {
		text: string;
		size: string;
		weight: string;
		checked?: boolean;
		disabled?: boolean;
		color?: string;
		required?: boolean;
	} = $props();
	const uid = $props.id();

	// Sanitize color to prevent CSS injection
	const safeColor = $derived(sanitizeColorValue(color || ''));

</script>

<div style={disabled ? 'opacity: 0.5; cursor: not-allowed;' : ''}>
	<label
		for="{uid}-checkbox"
		style="font-size: {size}; font-weight: {weight}; {safeColor
			? `color: ${safeColor};`
			: 'color: var(--text-primary);'} {disabled ? 'cursor: not-allowed;' : ''}"
		>{text}{required ? ' *' : ''}</label
	>
	<input
		id="{uid}-checkbox"
		type="checkbox"
		bind:checked
		{disabled}
		{required}
		style={disabled ? 'cursor: not-allowed;' : ''}
	/>
</div>
