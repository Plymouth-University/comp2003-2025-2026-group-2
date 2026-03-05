<script lang="ts">
	let {
		text = $bindable(''),
		size = 16,
		weight = 'normal',
		placeholder = 'Text Input',
		disabled = false,
		fontFamily = 'system-ui',
		textDecoration = 'none',
		color = '',
		required = false,
		maxLength = undefined as number | undefined,
		minLength = undefined as number | undefined,
		inputType = 'text'
	}: {
		text: string;
		size: number;
		weight: string;
		placeholder: string;
		disabled?: boolean;
		fontFamily?: string;
		textDecoration?: string;
		color?: string;
		required?: boolean;
		maxLength?: number;
		minLength?: number;
		inputType?: string;
	} = $props();

	function handleInput(e: Event) {
		const target = e.target as HTMLInputElement;
		let value = target.value;

		if (inputType === 'int') {
			value = value.replace(/[^0-9-]/g, '');
		} else if (inputType === 'float') {
			value = value.replace(/[^0-9.-]/g, '');
		}

		text = value;
	}

	// Ensure maxLength and minLength are valid non-negative integers
	const validMaxLength = $derived(
		maxLength !== undefined && maxLength >= 0 ? maxLength : undefined
	);
	const validMinLength = $derived(
		minLength !== undefined && minLength >= 0 ? minLength : undefined
	);
</script>

<input
	type="text"
	bind:value={text}
	oninput={handleInput}
	class={`
        ${weight === 'light' ? 'font-light' : ''}
        ${weight === 'normal' ? 'font-normal' : ''}
        ${weight === 'bold' ? 'font-bold' : ''}
        border-2 px-2 py-1
    `}
	{placeholder}
	{disabled}
	{required}
	maxlength={validMaxLength}
	minlength={validMinLength}
	style="border-color: var(--border-primary); {color
		? `color: ${color};`
		: ''} background-color: var(--bg-primary); font-size: {size}px; font-family: {fontFamily}; text-decoration: {textDecoration}; {disabled
		? 'opacity: 0.5; cursor: not-allowed;'
		: ''}"
/>
