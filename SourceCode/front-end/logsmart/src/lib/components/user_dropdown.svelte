<script lang="ts">
	let {
		selected = $bindable(''),
		options,
		disabled = false
	}: { selected: string; options: string[]; disabled?: boolean } = $props();
</script>

<div>
	<select
		bind:value={selected}
		{disabled}
		class="border-2 px-3 py-2 text-center text-xl font-medium"
		style="border-color: var(--border-primary); color: var(--text-primary); background-color: var(--bg-primary); {disabled
			? 'opacity: 0.5; cursor: not-allowed;'
			: ''}"
	>
		{#each options as option}
			<option value={option}>{option}</option>
		{/each}
	</select>
</div>
