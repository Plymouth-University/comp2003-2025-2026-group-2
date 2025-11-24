<script lang="ts">
    import TemperaturePicker from '$lib/components/temperature_picker.svelte';
	import LogButton1 from '$lib/components/next_button.svelte';
    import LogButton2 from '$lib/components/previous_button.svelte';

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

<style>
    .container {
        display:flex;
        flex-direction: row;
        flex-shrink:0;
        margin-top: 1%;
    }

    .left_element{
        position:fixed;
        margin-left:35%;
        padding-right:100px;
        min-width:220px;
    }

    .right_element{
        position:fixed;
        margin-left:50%;
        padding-left:100px;
        min-width:200px;
        
    }
    
</style>

<main>
    <h1 class="text-3xl font-bold text-center mb-6" style="padding-top:1%">Freezer Temperature</h1>
    <div class="border-2 rounded-lg bg-white p-8" style="border-color: #000100; margin-left:10%; margin-right:10%;">
        <p class="text-2xl font-bold text-left mb-6">Input the temperature of each freezer:</p>
        <div class="grid grid-cols-[120px_1fr_120px] gap-8 mb-4">
            <div class="text-center font-bold" style="color: #000100;">
                <!-- Empty for input column -->
            </div>
            <div class="font-bold text-lg" style="color: #000100;">
                Freezers
                <div class="h-0.5 bg-current mt-1"></div>
            </div>
            <div class="text-right font-bold text-lg" style="color: #000100;">
                Units
                <div class="h-0.5 bg-current mt-1"></div>
            </div>
        </div>

        
        <!-- Temperature Input Rows -->		
        <div class="space-y-6">
            {#each canvasItems as item}
            <TemperaturePicker {item} />
            {/each}
        </div>
    </div>
    <div class="container">
        <div class="left_element">
            <LogButton2/>
        </div>
        <div class="right_element">
            <LogButton1/>
        </div>
    </div>

</main>