<script lang="ts">
	let {
		editable = false,
		text = $bindable('Label Text'),
		size = 16,
		weight = 'normal'
	}: { editable: boolean; text: string; size: number; weight: string } = $props();

	let element: HTMLParagraphElement;

	function handleBlur() {
		text = element?.textContent ?? '';
	}
</script>

<p
	bind:this={element}
	contenteditable={editable}
	onblur={handleBlur}
	class={`
		${weight === 'light' ? 'font-light' : ''}
		${weight === 'normal' ? 'font-normal' : ''}
		${weight === 'bold' ? 'font-bold' : ''}
		${editable ? 'cursor-text outline-none' : ''}
	`}
	style="color: var(--text-primary); font-size: {size}px;"
>
	{text}
</p>
