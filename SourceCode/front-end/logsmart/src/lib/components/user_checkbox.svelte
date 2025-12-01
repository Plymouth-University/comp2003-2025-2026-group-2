<script lang="ts">
    let { text, size, weight } = $props<{ children: any; }>();

    let checked = $state(false);
</script>
<div>
    <label for="checkbox" style="font-size: {size}; font-weight: {weight};">{text}</label>
    <input id="checkbox" type="checkbox" bind:checked={checked}/>
</div>