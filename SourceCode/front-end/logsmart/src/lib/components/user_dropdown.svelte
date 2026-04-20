<script lang="ts">
	let {
		selected = $bindable(''),
		options,
		disabled = false,
		placeholder = ''
	}: { selected: string; options: string[]; disabled?: boolean; placeholder?: string } = $props();
</script>

<div>
	<select
		bind:value={selected}
		{disabled}
		class="border-2 px-3 py-2 text-center text-sm font-medium"
		style="border-color: var(--border-primary); color: var(--text-primary); background-color: var(--bg-primary); {disabled
			? 'opacity: 0.5;'
			: ''}"
	>
		{#if placeholder}
			<option value="" disabled>{placeholder}</option>
		{/if}
		{#each options as option (option)}
			<option value={option}>{option}</option>
		{/each}
	</select>
</div>
