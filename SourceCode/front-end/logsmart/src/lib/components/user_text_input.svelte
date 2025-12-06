<script lang="ts">
	let {
		text = $bindable(''),
		size = 16,
		weight = 'normal',
		placeholder = 'Text Input',
		disabled = false
	}: {
		text: string;
		size: number;
		weight: string;
		placeholder: string;
		disabled?: boolean;
	} = $props();
</script>

<input
	type="text"
	bind:value={text}
	class={`
        ${weight === 'light' ? 'font-light' : ''}
        ${weight === 'normal' ? 'font-normal' : ''}
        ${weight === 'bold' ? 'font-bold' : ''}
        border-2 px-2 py-1
    `}
	{placeholder}
	{disabled}
	style="border-color: var(--border-primary); color: var(--text-primary); background-color: var(--bg-primary); font-size: {size}px; {disabled
		? 'opacity: 0.5; cursor: not-allowed;'
		: ''}"
/>
