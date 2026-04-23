<script lang="ts">
	import type { Snippet } from 'svelte';

	let {
		title,
		hasItems,
		emptyMessage,
		children
	}: {
		title: string;
		hasItems: boolean;
		emptyMessage: string;
		children?: Snippet;
	} = $props();
</script>

<div class="mb-8">
	<h2 class="mb-4 text-3xl font-bold text-text-primary">{title}</h2>
	{#if hasItems}
		<div class="space-y-2">
			{@render children?.()}
		</div>
	{:else}
		<div class="rounded bg-bg-primary p-6 text-center text-text-secondary">
			{emptyMessage}
		</div>
	{/if}
</div>
