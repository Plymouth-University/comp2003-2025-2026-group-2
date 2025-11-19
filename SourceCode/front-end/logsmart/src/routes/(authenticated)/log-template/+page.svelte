<script lang="ts">
    let canvasItems = $state([
        { id: 1, value: 7, label: 'Freezer 1', unit: '°C'},
        { id: 2, value: 0, label: 'Freezer 2', unit: '°C'},
        { id: 3, value: -15, label: 'Freezer 3', unit: '°C'}
    ])
</script>

<style>
    /* Remove default number input arrows */
	input[type="number"]::-webkit-inner-spin-button,
	input[type="number"]::-webkit-outer-spin-button {
		-webkit-appearance: none;
		margin: 0;
	}
	
	input[type="number"] {
		-moz-appearance: textfield;
		appearance: textfield;
	}
</style>

<main>
    <h1>Freezer Temperature</h1>
    <p>Input freezer temperatures.</p>
    <h2>Input data:</h2>
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
            <div class="grid grid-cols-[120px_1fr_120px] gap-8 items-center">
                <!-- Input with buttons -->
                <div class="flex items-center gap-3">
                    <input 
                        type="number" 
                        bind:value={item.value}
						class="w-16 px-3 py-2 text-center text-xl font-medium border-2"
					    style="border-color: #000100; color: #000100;"
                    />
                    <div class="flex flex-col gap-1">
                        <button 
                            type="button"
                            onclick={() => item.value++}
                            class="w-7 h-7 border-2 flex items-center justify-center hover:bg-gray-100 transition-all hover:scale-110 active:scale-95 rounded-sm"
                            style="border-color: #000100; color: #000100;"
                            aria-label="Increase temperature"
                        >
                            <svg width="14" height="14" viewBox="0 0 14 14" fill="none" stroke="currentColor" stroke-width="2.5">
                                <polyline points="2 8 7 3 12 8"></polyline>
                            </svg>
                        </button>
                        <button
                            type="button"
                            onclick={() => item.value--}
                            class="w-7 h-7 border-2 flex items-center justify-center hover:bg-gray-100 transition-all hover:scale-110 active:scale-95 rounded-sm"
                            style="border-color: #000100; color: #000100;"
                            aria-label="Decrease temperature"
                        >
                            <svg width="14" height="14" viewBox="0 0 14 14" fill="none" stroke="currentColor" stroke-width="2.5">
                                <polyline points="2 6 7 11 12 6"></polyline>
                            </svg>
                        </button>
                    </div>
			    </div>
								
				<!-- Name/Label -->
				<div class="text-lg" style="color: #000100;">
                    {item.label}
                </div>			
				<!-- Units -->
				<div class="text-right text-lg" style="color: #000100;">
                    {item.unit}
				</div>
			</div>
		{/each}
	</div>
</main>