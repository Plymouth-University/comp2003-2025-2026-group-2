<script lang="ts">
	let {
		editable = false,
		text = $bindable('Label Text'),
		size = 16,
		weight = 'normal',
		fontFamily = 'system-ui',
		textDecoration = 'none',
		color = ''
	}: {
		editable: boolean;
		text: string;
		size: number;
		weight: string;
		fontFamily?: string;
		textDecoration?: string;
		color?: string;
	} = $props();

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
	style="{color
		? `color: ${color};`
		: 'color: var(--text-primary);'} font-size: {size}px; font-family: {fontFamily}; text-decoration: {textDecoration};"
>
	{text}
</p>
