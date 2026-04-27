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
		class="border-2 border-border-primary bg-bg-secondary px-3 py-2 text-center text-sm font-medium text-text-primary"
		style={disabled ? 'opacity: 0.5;' : ''}
	>
		{#if placeholder}
			<option value="" disabled>{placeholder}</option>
		{/if}
		{#each options as option (option)}
			<option value={option}>{option}</option>
		{/each}
	</select>
</div>
