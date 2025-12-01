<script lang="ts">
	let {
		text = '',
		size = 'medium',
		weight = 'normal',
		placeholder = 'Text Input'
	}: { text: string; size: string; weight: string; placeholder: string } = $props();
</script>

<input
	type="text"
	bind:value={text}
	class={`
        ${size === 'small' ? 'text-sm' : ''}
        ${size === 'medium' ? 'text-base' : ''}
        ${size === 'large' ? 'text-lg' : ''}
        ${weight === 'light' ? 'font-light' : ''}
        ${weight === 'normal' ? 'font-normal' : ''}
        ${weight === 'bold' ? 'font-bold' : ''}
        border-2 px-2 py-1
    `}
	{placeholder}
	disabled
	style="border-color: #000100; color: #000100;"
/>
