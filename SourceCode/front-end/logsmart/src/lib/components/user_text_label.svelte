<script lang="ts">
	let {
		editable = false,
		text = '',
		size = 'medium',
		weight = 'normal'
	}: { editable: boolean; text: string; size: string; weight: string } = $props();
</script>

{#if editable}
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
		style="border-color: #000100; color: #000100;"
	/>
{:else}
	<p
		class={`
    ${size === 'small' ? 'text-sm' : ''}
    ${size === 'medium' ? 'text-base' : ''}
    ${size === 'large' ? 'text-lg' : ''}
    ${weight === 'light' ? 'font-light' : ''}
    ${weight === 'normal' ? 'font-normal' : ''}
    ${weight === 'bold' ? 'font-bold' : ''}
`}
	>
		{text}
	</p>
{/if}
