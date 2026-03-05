<script lang="ts">
	import { sanitizeColorValue } from '$lib/utils/validation';

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

	// Sanitize color to prevent CSS injection
	const safeColor = $derived(sanitizeColorValue(color || ''));
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
	style="{safeColor
		? `color: ${safeColor};`
		: 'color: var(--text-primary);'} font-size: {size}px; font-family: {fontFamily}; text-decoration: {textDecoration};"
>
	{text}
</p>
