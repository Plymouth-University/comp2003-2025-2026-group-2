<script lang="ts">
	import TemperaturePicker from '$lib/components/temperature_picker.svelte';
	import NextButton from './next_button.svelte';
	import PreviousButton from './previous_button.svelte';

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
	<h1 class="mb-6 text-center text-3xl font-bold" style="padding-top:1%">Freezer Temperature</h1>
	<div
		class="rounded-lg border-2 bg-white p-8"
		style="border-color: #000100; margin-left:10%; margin-right:10%;"
	>
		<p class="mb-6 text-left text-2xl font-bold">Input the temperature of each freezer:</p>
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
				<TemperaturePicker
					value={item.value}
					min={-10}
					max={10}
					label={item.label}
					unit={item.unit}
				/>
			{/each}
		</div>
	</div>
	<div class="container">
		<div class="left_element">
			<PreviousButton />
		</div>
		<div class="right_element">
			<NextButton />
		</div>
	</div>
</main>

<style>
	.container {
		display: flex;
		flex-direction: row;
		flex-shrink: 0;
		margin-top: 1%;
	}

	.left_element {
		position: fixed;
		margin-left: 35%;
		padding-right: 100px;
		min-width: 220px;
	}

	.right_element {
		position: fixed;
		margin-left: 50%;
		padding-left: 100px;
		min-width: 200px;
	}
</style>
