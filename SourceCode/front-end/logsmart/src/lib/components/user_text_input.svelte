<script lang="ts">
	let {
		text = '',
		size = 16,
		weight = 'normal',
		placeholder = 'Text Input'
	}: { text: string; size: number; weight: string; placeholder: string } = $props();
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
	disabled
	style="border-color: #000100; color: #000100; font-size: {size}px;"
/>
