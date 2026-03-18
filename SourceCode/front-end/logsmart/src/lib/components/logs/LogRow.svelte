<script lang="ts">
	import type { Snippet } from 'svelte';

	let props: {
		title: string;
		titleAttr?: string;
		meta?: Snippet;
		actions?: Snippet;
		children?: Snippet;
	} = $props();
	const resolvedTitleAttr = $derived(props.titleAttr ?? props.title);
</script>

<div
	class="flex items-center justify-between rounded border-2 p-4"
	style="background-color: var(--bg-primary); border-color: var(--border-primary);"
>
	<div class="min-w-0 flex-1">
		<div
			class="text-ellipsis overflow-hidden text-lg font-semibold whitespace-nowrap lg:text-xl"
			style="color: var(--text-primary);"
			title={resolvedTitleAttr}
		>
			{props.title}
		</div>
		<div class="text-sm" style="color: var(--text-secondary);">
			{@render props.meta?.()}
		</div>
	</div>
	<div class="flex gap-2">
		{@render props.actions?.()}
	</div>
	{@render props.children?.()}
</div>