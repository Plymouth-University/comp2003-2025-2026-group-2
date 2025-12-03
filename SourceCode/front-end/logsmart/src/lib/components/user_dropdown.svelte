<script lang="ts">
	let { selected = $bindable(''), options }: { selected: string; options: string[] } = $props();
</script>

<div>
	<select
		bind:value={selected}
		class="border-2 px-3 py-2 text-center text-xl font-medium"
		style="border-color: var(--border-primary); color: var(--text-primary); background-color: var(--bg-primary);"
	>
		{#each options as option}
			<option value={option}>{option}</option>
		{/each}
	</select>
</div>
