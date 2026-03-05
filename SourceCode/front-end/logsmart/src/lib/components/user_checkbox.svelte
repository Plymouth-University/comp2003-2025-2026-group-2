<script lang="ts">
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
</script>

<div style={disabled ? 'opacity: 0.5; cursor: not-allowed;' : ''}>
	<label
		for="{uid}-checkbox"
		style="font-size: {size}; font-weight: {weight}; {color
			? `color: ${color};`
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
