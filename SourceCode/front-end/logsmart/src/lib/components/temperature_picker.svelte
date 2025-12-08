<script lang="ts">
	let {
		value = $bindable(0),
		min = -20,
		max = 50,
		label = 'Temperature',
		unit = '°C',
		disabled = false
	}: {
		value: number;
		min: number;
		max: number;
		label: string;
		unit: string;
		disabled?: boolean;
	} = $props();

	const increaseValue = () => {
		if (value < max) {
			value += 1;
		}
	};
	const decreaseValue = () => {
		if (value > min) {
			value -= 1;
		}
	};
	const handleInputChange = (event: Event) => {
		const input = event.target as HTMLInputElement;
		let newValue = parseInt(input.value);
		if (isNaN(newValue)) {
			newValue = min;
		}
		if (newValue < min) {
			newValue = min;
		} else if (newValue > max) {
			newValue = max;
		}
		value = newValue;
	};
</script>

<div class="grid grid-cols-[120px_1fr_120px] items-center gap-8">
	<div class="flex items-center gap-3">
		<input
			type="number"
			bind:value
			onchange={handleInputChange}
			max={max}
			min={min}
			{disabled}
			class="w-16 border-2 px-3 py-2 text-center text-xl font-medium"
			style="border-color: var(--border-primary); color: var(--text-primary); background-color: var(--bg-primary); {disabled
				? 'opacity: 0.5; cursor: not-allowed;'
				: ''}"
		/>
		<div class="flex flex-col gap-1">
			<button
				type="button"
				onclick={increaseValue}
				{disabled}
				class="flex h-7 w-7 items-center justify-center rounded-sm border-2 transition-all
                {disabled ? '' : 'hover:scale-110 hover:bg-gray-100 active:scale-95'}"
				style="border-color: var(--border-primary); color: var(--text-primary); background-color: var(--bg-primary); {disabled
					? 'opacity: 0.5; cursor: not-allowed;'
					: ''}"
				aria-label="Increase temperature"
			>
				<svg
					width="14"
					height="14"
					viewBox="0 0 14 14"
					fill="none"
					stroke="currentColor"
					stroke-width="2.5"
				>
					<polyline points="2 8 7 3 12 8"></polyline>
				</svg>
			</button>
			<button
				type="button"
				onclick={decreaseValue}
				{disabled}
				class="flex h-7 w-7 items-center justify-center rounded-sm border-2 transition-all {disabled
					? ''
					: 'hover:scale-110 hover:bg-gray-100 active:scale-95'}"
				style="border-color: var(--border-primary); color: var(--text-primary); background-color: var(--bg-primary); {disabled
					? 'opacity: 0.5; cursor: not-allowed;'
					: ''}"
				aria-label="Decrease temperature"
			>
				<svg
					width="14"
					height="14"
					viewBox="0 0 14 14"
					fill="none"
					stroke="currentColor"
					stroke-width="2.5"
				>
					<polyline points="2 6 7 11 12 6"></polyline>
				</svg>
			</button>
		</div>
	</div>

	<!-- Name/Label -->
	<div class="text-lg" style="color: var(--text-primary);">
		{label}
	</div>
	<!-- Units -->
	<div class="text-right text-lg" style="color: var(--text-primary);">
		{unit}
	</div>
</div>

<style>
	/* Remove default number input arrows */
	input[type='number']::-webkit-inner-spin-button,
	input[type='number']::-webkit-outer-spin-button {
		-webkit-appearance: none;
		margin: 0;
	}

	input[type='number'] {
		-moz-appearance: textfield;
		appearance: textfield;
	}
</style>
