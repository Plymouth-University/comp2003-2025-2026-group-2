<script lang="ts">
	import TemperaturePicker from '$lib/components/temperature_picker.svelte';

	let canvasItems = $state([
		{
			id: 1,
			value: 7,
			label: 'Freezer 1',
			unit: '°C'
		},
		{
			id: 2,
			value: 0,
			label: 'Freezer 2',
			unit: '°C'
		},
		{
			id: 3,
			value: -15,
			label: 'Freezer 3',
			unit: '°C'
		}
	]);
</script>

<main>
	<h1>Freezer Temperature</h1>
	<p>Input freezer temperatures.</p>
	<h2>Input data:</h2>
	<div class="mb-4 grid grid-cols-[120px_1fr_120px] gap-8">
		<div class="text-center font-bold" style="color: #000100;">
			<!-- Empty for input column -->
		</div>
		<div class="text-lg font-bold" style="color: #000100;">
			Freezers
			<div class="mt-1 h-0.5 bg-current"></div>
		</div>
		<div class="text-right text-lg font-bold" style="color: #000100;">
			Units
			<div class="mt-1 h-0.5 bg-current"></div>
		</div>
	</div>

	<!-- Temperature Input Rows -->
	<div class="space-y-6">
		{#each canvasItems as item}
			<TemperaturePicker {item} />
		{/each}
	</div>
</main>
