<script lang="ts">
	let { selected = $bindable(''), options }: { selected: string; options: string[] } = $props();
</script>

<div>
	<select
		bind:value={selected}
		class="border-2 px-3 py-2 text-center text-xl font-medium"
		style="border-color: #000100; color: #000100;"
	>
		{#each options as option}
			<option value={option}>{option}</option>
		{/each}
	</select>
</div>
