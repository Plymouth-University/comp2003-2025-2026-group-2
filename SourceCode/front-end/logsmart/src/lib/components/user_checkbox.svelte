<script lang="ts">
	let {
		text = 'Checkbox Label',
		size = '16px',
		weight = 'normal'
	}: { text: string; size: string; weight: string } = $props();
	const uid = $props.id();

	let checked = $state(false);
</script>

<div>
	<label for="{uid}-checkbox" style="font-size: {size}; font-weight: {weight}; color: var(--text-primary);">{text}</label>
	<input id="{uid}-checkbox" type="checkbox" bind:checked />
</div>
