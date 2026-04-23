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
	class="flex items-center justify-between rounded border-2 border-border-primary bg-bg-primary p-4"
>
	<div class="min-w-0 flex-1">
		<div
			class="overflow-hidden text-lg font-semibold text-ellipsis whitespace-nowrap text-text-primary lg:text-xl"
			title={resolvedTitleAttr}
		>
			{props.title}
		</div>
		<div class="text-sm text-text-secondary">
			{@render props.meta?.()}
		</div>
	</div>
	<div class="flex gap-2">
		{@render props.actions?.()}
	</div>
	{@render props.children?.()}
</div>
