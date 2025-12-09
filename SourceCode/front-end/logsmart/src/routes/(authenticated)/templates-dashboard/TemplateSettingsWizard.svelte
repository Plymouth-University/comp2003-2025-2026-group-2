<script lang="ts">
	import type { Template, ScheduleFrequency, DayOfWeek, TemplateSchedule } from './types';

	let {
		template,
		isOpen = $bindable(false),
		onSave
	}: {
		template: Template;
		isOpen: boolean;
		onSave: (schedule: TemplateSchedule) => void;
	} = $props();

	const frequencyOptions: { value: ScheduleFrequency; label: string }[] = [
		{ value: 'daily', label: 'Daily' },
		{ value: 'weekly', label: 'Weekly' },
		{ value: 'monthly', label: 'Monthly' },
		{ value: 'yearly', label: 'Yearly' },
		{ value: 'custom', label: 'Custom Interval' }
	];

	const daysOfWeek: { value: DayOfWeek; label: string; short: string }[] = [
		{ value: 'monday', label: 'Monday', short: 'Mon' },
		{ value: 'tuesday', label: 'Tuesday', short: 'Tue' },
		{ value: 'wednesday', label: 'Wednesday', short: 'Wed' },
		{ value: 'thursday', label: 'Thursday', short: 'Thu' },
		{ value: 'friday', label: 'Friday', short: 'Fri' },
		{ value: 'saturday', label: 'Saturday', short: 'Sat' },
		{ value: 'sunday', label: 'Sunday', short: 'Sun' }
	];

	let frequency = $derived<ScheduleFrequency>(template.schedule.frequency);
	let selectedDays = $derived<DayOfWeek[]>(template.schedule.daysOfWeek || []);
	let weeklyDay = $derived<DayOfWeek>(template.schedule.dayOfWeek || 'monday');
	let dayOfMonth = $derived(template.schedule.dayOfMonth || 1);
	let monthOfYear = $derived(template.schedule.monthOfYear || 1);
	let customIntervalDays = $derived(template.schedule.customIntervalDays || 7);

	function toggleDay(day: DayOfWeek) {
		if (selectedDays.includes(day)) {
			selectedDays = selectedDays.filter((d) => d !== day);
		} else {
			selectedDays = [...selectedDays, day];
		}
	}

	function handleSave() {
		const schedule: TemplateSchedule = { frequency };

		if (frequency === 'daily') {
			schedule.daysOfWeek = selectedDays.length > 0 ? selectedDays : undefined;
		} else if (frequency === 'weekly') {
			schedule.dayOfWeek = weeklyDay;
		} else if (frequency === 'monthly') {
			schedule.dayOfMonth = dayOfMonth;
		} else if (frequency === 'yearly') {
			schedule.dayOfMonth = dayOfMonth;
			schedule.monthOfYear = monthOfYear;
		} else if (frequency === 'custom') {
			schedule.customIntervalDays = customIntervalDays;
		}

		onSave(schedule);
		isOpen = false;
	}

	function handleClose() {
		isOpen = false;
	}

	function handleBackdropClick(e: MouseEvent) {
		if (e.target === e.currentTarget) {
			handleClose();
		}
	}

	const months = [
		'January',
		'February',
		'March',
		'April',
		'May',
		'June',
		'July',
		'August',
		'September',
		'October',
		'November',
		'December'
	];
</script>

{#if isOpen}
	<div
		class="fixed inset-0 z-50 flex items-center justify-center bg-black/50"
		onclick={handleBackdropClick}
		onkeydown={(e) => e.key === 'Escape' && handleClose()}
		role="dialog"
		aria-modal="true"
		aria-labelledby="wizard-title"
		tabindex="-1"
	>
		<div
			class="w-full max-w-lg rounded-lg border-2 shadow-xl"
			style="background-color: var(--bg-primary); border-color: var(--border-primary);"
		>
			<div class="border-b-2 px-6 py-4" style="border-color: var(--border-primary);">
				<h2 id="wizard-title" class="text-xl font-bold" style="color: var(--text-primary);">
					Schedule Settings: {template.name}
				</h2>
			</div>

			<div class="space-y-6 px-6 py-6">
				<div>
					<label
						for="frequency-select"
						class="mb-2 block font-medium"
						style="color: var(--text-secondary);"
					>
						Frequency
					</label>
					<select
						id="frequency-select"
						bind:value={frequency}
						class="w-full rounded border-2 px-4 py-2"
						style="border-color: var(--border-primary); background-color: var(--bg-secondary); color: var(--text-primary);"
					>
						{#each frequencyOptions as option (option.value)}
							<option value={option.value}>{option.label}</option>
						{/each}
					</select>
				</div>

				{#if frequency === 'daily'}
					<div>
						<p class="mb-3 font-medium" style="color: var(--text-secondary);">
							Select days when this template should be completed:
						</p>
						<p class="mb-3 text-sm" style="color: var(--text-secondary);">
							Leave all unselected for every day
						</p>
						<div class="flex flex-wrap gap-2">
							{#each daysOfWeek as day (day.value)}
								<button
									type="button"
									class="rounded border-2 px-3 py-2 font-medium transition-colors"
									class:day-selected={selectedDays.includes(day.value)}
									class:day-unselected={!selectedDays.includes(day.value)}
									onclick={() => toggleDay(day.value)}
								>
									{day.short}
								</button>
							{/each}
						</div>
					</div>
				{/if}

				{#if frequency === 'weekly'}
					<div>
						<p class="mb-3 font-medium" style="color: var(--text-secondary);">
							Select which day of the week this template should be completed on:
						</p>
						<div class="flex flex-wrap gap-2">
							{#each daysOfWeek as day (day.value)}
								<button
									type="button"
									class="rounded border-2 px-3 py-2 font-medium transition-colors"
									class:day-selected={weeklyDay === day.value}
									class:day-unselected={weeklyDay !== day.value}
									onclick={() => (weeklyDay = day.value)}
								>
									{day.short}
								</button>
							{/each}
						</div>
					</div>
				{/if}

				{#if frequency === 'monthly'}
					<div>
						<label
							for="monthly-day"
							class="mb-2 block font-medium"
							style="color: var(--text-secondary);"
						>
							Day of the month it will become available from
						</label>
						<input
							id="monthly-day"
							type="number"
							min="1"
							max="31"
							bind:value={dayOfMonth}
							class="w-full rounded border-2 px-4 py-2"
							style="border-color: var(--border-primary); background-color: var(--bg-secondary); color: var(--text-primary);"
						/>
					</div>
				{/if}

				{#if frequency === 'yearly'}
					<div>
						<label
							for="yearly-date"
							class="mb-2 block font-medium"
							style="color: var(--text-secondary);"
						>
							Date of year it will become available from
						</label>
						<input
							id="yearly-date"
							type="date"
							value={`2024-${String(monthOfYear).padStart(2, '0')}-${String(dayOfMonth).padStart(2, '0')}`}
							onchange={(e) => {
								const date = new Date(e.currentTarget.value);
								monthOfYear = date.getMonth() + 1;
								dayOfMonth = date.getDate();
							}}
							class="w-full rounded border-2 px-4 py-2"
							style="border-color: var(--border-primary); background-color: var(--bg-secondary); color: var(--text-primary);"
						/>
						<p class="mt-1 text-sm" style="color: var(--text-secondary);">
							Selected: {months[monthOfYear - 1]}
							{dayOfMonth}
						</p>
					</div>
				{/if}

				{#if frequency === 'custom'}
					<div>
						<label
							for="custom-interval"
							class="mb-2 block font-medium"
							style="color: var(--text-secondary);"
						>
							Repeat every X days
						</label>
						<input
							id="custom-interval"
							type="number"
							min="1"
							max="365"
							bind:value={customIntervalDays}
							class="w-full rounded border-2 px-4 py-2"
							style="border-color: var(--border-primary); background-color: var(--bg-secondary); color: var(--text-primary);"
						/>
					</div>
				{/if}
			</div>

			<div
				class="flex justify-end gap-3 border-t-2 px-6 py-4"
				style="border-color: var(--border-primary);"
			>
				<button
					type="button"
					class="btn-cancel rounded px-4 py-2 font-medium"
					onclick={handleClose}
				>
					Cancel
				</button>
				<button
					type="button"
					class="btn-save rounded px-4 py-2 font-medium text-white"
					onclick={handleSave}
				>
					Save
				</button>
			</div>
		</div>
	</div>
{/if}

<style>
	.day-selected {
		background-color: #94c5cc;
		border-color: #94c5cc;
		color: white;
	}

	.day-unselected {
		background-color: var(--bg-secondary);
		border-color: var(--border-primary);
		color: var(--text-primary);
	}

	.day-unselected:hover {
		background-color: var(--bg-primary);
		border-color: #94c5cc;
	}

	.btn-cancel {
		background-color: var(--bg-secondary);
		color: var(--text-primary);
		border: 2px solid var(--border-primary);
		transition: background-color 0.15s ease;
	}

	.btn-cancel:hover {
		background-color: var(--bg-primary);
	}

	.btn-save {
		background-color: #5cb85c;
		transition: background-color 0.15s ease;
	}

	.btn-save:hover {
		background-color: #449d44;
	}

	.btn-save:active {
		background-color: #398439;
	}
</style>
