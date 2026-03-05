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
	maxlength={maxLength}
	style="border-color: var(--border-primary); {color
		? `color: ${color};`
		: ''} background-color: var(--bg-primary); font-size: {size}px; font-family: {fontFamily}; text-decoration: {textDecoration}; {disabled
		? 'opacity: 0.5; cursor: not-allowed;'
		: ''}"
/>
