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
	<h2 class="mb-4 text-3xl font-bold" style="color: var(--text-primary);">{title}</h2>
	{#if hasItems}
		<div class="space-y-2">
			{@render children?.()}
		</div>
	{:else}
		<div
			class="rounded p-6 text-center"
			style="background-color: var(--bg-primary); color: var(--text-secondary);"
		>
			{emptyMessage}
		</div>
	{/if}
</div>
