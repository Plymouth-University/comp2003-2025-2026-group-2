<script lang="ts">
	let { text, size, weight }: { text: string; size: string; weight: string } = $props<{ children: any }>();
    const uid = $props.id();

	let checked = $state(false);
</script>

<div>
	<label for="{uid}-checkbox" style="font-size: {size}; font-weight: {weight};">{text}</label>
	<input id="{uid}-checkbox" type="checkbox" bind:checked />
</div>
