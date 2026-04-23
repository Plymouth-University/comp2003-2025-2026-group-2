<script lang="ts">
	import { onMount } from 'svelte';

	type PickerView = 'day' | 'month' | 'year';

	let {
		value = $bindable(''),
		placeholder = 'DD/MM/YYYY',
		inputId,
		maxDate = null,
		openCalendarAriaLabel = 'Open calendar',
		inputClass = 'flex-1 border-2 px-4 py-2',
		inputStyle = 'border-color: var(--border-primary); background-color: var(--bg-primary); color: var(--text-primary);',
		buttonClass = 'transform border-2 p-2 transition-all duration-150 hover:scale-105',
		buttonStyle = 'border-color: var(--border-primary); background-color: var(--bg-primary); color: var(--text-primary);',
		pickerStyle = 'border-color: var(--border-primary); background-color: var(--bg-primary); min-width: 280px; overflow: hidden;',
		onValueInput = (_nextValue: string): void => {},
		onValueBlur = (): void => {}
	}: {
		value: string;
		placeholder?: string;
		inputId?: string;
		maxDate?: Date | null;
		openCalendarAriaLabel?: string;
		inputClass?: string;
		inputStyle?: string;
		buttonClass?: string;
		buttonStyle?: string;
		pickerStyle?: string;
		onValueInput?: (nextValue: string) => void;
		onValueBlur?: () => void;
	} = $props();

	let isOpen = $state(false);
	let pickerView = $state<PickerView>('day');
	let slideDirection = $state<'left' | 'right'>('left');
	let viewDate = $state(new Date());
	let container: HTMLDivElement | null = null;

	const monthNames = [
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

	const monthNamesShort = [
		'Jan',
		'Feb',
		'Mar',
		'Apr',
		'May',
		'Jun',
		'Jul',
		'Aug',
		'Sep',
		'Oct',
		'Nov',
		'Dec'
	];

	const maxDateNormalized = $derived(maxDate ? normalizeDate(maxDate) : null);

	onMount(() => {
		const handleDocumentPointerDown = (event: PointerEvent): void => {
			if (!isOpen) return;

			const target = event.target as Node | null;
			if (!target) return;

			if (container && !container.contains(target)) {
				isOpen = false;
			}
		};

		document.addEventListener('pointerdown', handleDocumentPointerDown);
		return () => {
			document.removeEventListener('pointerdown', handleDocumentPointerDown);
		};
	});

	function normalizeDate(date: Date): Date {
		return new Date(date.getFullYear(), date.getMonth(), date.getDate());
	}

	function parseDisplayDate(displayValue: string): Date | null {
		const parts = displayValue.split('/');
		if (parts.length !== 3) return null;

		const [dayStr, monthStr, yearStr] = parts;
		const day = Number(dayStr);
		const month = Number(monthStr);
		const year = Number(yearStr);

		if (!Number.isInteger(day) || !Number.isInteger(month) || !Number.isInteger(year)) {
			return null;
		}

		const parsed = new Date(year, month - 1, day);
		if (
			parsed.getFullYear() !== year ||
			parsed.getMonth() !== month - 1 ||
			parsed.getDate() !== day
		) {
			return null;
		}

		return parsed;
	}

	function formatDisplayDate(date: Date): string {
		const day = String(date.getDate()).padStart(2, '0');
		const month = String(date.getMonth() + 1).padStart(2, '0');
		const year = date.getFullYear();
		return `${day}/${month}/${year}`;
	}

	function getDaysInMonth(date: Date): number {
		return new Date(date.getFullYear(), date.getMonth() + 1, 0).getDate();
	}

	function getFirstDayOfMonth(date: Date): number {
		return new Date(date.getFullYear(), date.getMonth(), 1).getDay();
	}

	function getCalendarDays(date: Date): Array<number | null> {
		const daysInMonth = getDaysInMonth(date);
		const firstDay = getFirstDayOfMonth(date);
		const days: Array<number | null> = [];

		for (let i = 0; i < firstDay; i += 1) {
			days.push(null);
		}

		for (let day = 1; day <= daysInMonth; day += 1) {
			days.push(day);
		}

		return days;
	}

	function isAfterMax(date: Date): boolean {
		return maxDateNormalized ? normalizeDate(date).getTime() > maxDateNormalized.getTime() : false;
	}

	function resetPickerState(): void {
		pickerView = 'day';
		slideDirection = 'left';

		const selected = parseDisplayDate(value);
		if (selected) {
			viewDate = selected;
			return;
		}

		if (maxDateNormalized) {
			viewDate = maxDateNormalized;
			return;
		}

		viewDate = normalizeDate(new Date());
	}

	function togglePicker(): void {
		const willOpen = !isOpen;
		isOpen = willOpen;

		if (willOpen) {
			resetPickerState();
		}
	}

	function switchToMonthView(): void {
		slideDirection = 'right';
		pickerView = 'month';
	}

	function switchToYearView(): void {
		slideDirection = 'right';
		pickerView = 'year';
	}

	function switchToDayView(): void {
		slideDirection = 'left';
		pickerView = 'day';
	}

	function canGoToNextMonth(): boolean {
		if (!maxDateNormalized) return true;

		const viewYear = viewDate.getFullYear();
		const viewMonth = viewDate.getMonth();
		const maxYear = maxDateNormalized.getFullYear();
		const maxMonth = maxDateNormalized.getMonth();

		return viewYear < maxYear || (viewYear === maxYear && viewMonth < maxMonth);
	}

	function canGoToNextYear(): boolean {
		if (!maxDateNormalized) return true;
		return viewDate.getFullYear() < maxDateNormalized.getFullYear();
	}

	function canGoToNextYearRange(): boolean {
		if (!maxDateNormalized) return true;
		const currentStart = Math.floor(viewDate.getFullYear() / 12) * 12;
		return currentStart + 12 <= maxDateNormalized.getFullYear();
	}

	function previousMonth(): void {
		viewDate = new Date(viewDate.getFullYear(), viewDate.getMonth() - 1, 1);
	}

	function nextMonth(): void {
		if (!canGoToNextMonth()) return;
		viewDate = new Date(viewDate.getFullYear(), viewDate.getMonth() + 1, 1);
	}

	function previousYear(): void {
		viewDate = new Date(viewDate.getFullYear() - 1, viewDate.getMonth(), 1);
	}

	function nextYear(): void {
		if (!canGoToNextYear()) return;
		viewDate = new Date(viewDate.getFullYear() + 1, viewDate.getMonth(), 1);
	}

	function previousYearRange(): void {
		viewDate = new Date(viewDate.getFullYear() - 12, viewDate.getMonth(), 1);
	}

	function nextYearRange(): void {
		if (!canGoToNextYearRange()) return;
		viewDate = new Date(viewDate.getFullYear() + 12, viewDate.getMonth(), 1);
	}

	function getYearRange(): number[] {
		const currentYear = viewDate.getFullYear();
		const startYear = Math.floor(currentYear / 12) * 12;
		const years: number[] = [];
		for (let i = 0; i < 12; i += 1) {
			years.push(startYear + i);
		}
		return years;
	}

	function isMonthDisabled(monthIndex: number): boolean {
		if (!maxDateNormalized) return false;

		const viewYear = viewDate.getFullYear();
		const maxYear = maxDateNormalized.getFullYear();
		const maxMonth = maxDateNormalized.getMonth();

		if (viewYear > maxYear) return true;
		if (viewYear < maxYear) return false;
		return monthIndex > maxMonth;
	}

	function isYearDisabled(year: number): boolean {
		if (!maxDateNormalized) return false;
		return year > maxDateNormalized.getFullYear();
	}

	function selectDay(day: number): void {
		const selectedDate = new Date(viewDate.getFullYear(), viewDate.getMonth(), day);
		if (isAfterMax(selectedDate)) return;

		value = formatDisplayDate(selectedDate);
		onValueInput(value);
		isOpen = false;
	}

	function selectMonth(monthIndex: number): void {
		if (isMonthDisabled(monthIndex)) return;
		viewDate = new Date(viewDate.getFullYear(), monthIndex, 1);
		switchToDayView();
	}

	function selectYear(year: number): void {
		if (isYearDisabled(year)) return;
		viewDate = new Date(year, viewDate.getMonth(), 1);
		switchToMonthView();
	}

	function isSelectedDay(day: number): boolean {
		const currentSelected = parseDisplayDate(value);
		if (!currentSelected) return false;

		return (
			currentSelected.getDate() === day &&
			currentSelected.getMonth() === viewDate.getMonth() &&
			currentSelected.getFullYear() === viewDate.getFullYear()
		);
	}

	function handleTextInput(event: Event): void {
		const input = event.currentTarget as HTMLInputElement;
		value = input.value;
		onValueInput(value);
	}

	function handleBlur(): void {
		onValueBlur();
	}
</script>

<div class="relative" bind:this={container}>
	<div class="flex items-center gap-2">
		<input
			id={inputId}
			type="text"
			bind:value
			oninput={handleTextInput}
			onblur={handleBlur}
			{placeholder}
			class={inputClass}
			style={inputStyle}
		/>
		<button
			type="button"
			onclick={togglePicker}
			aria-label={openCalendarAriaLabel}
			class={buttonClass}
			style={buttonStyle}
		>
			<svg
				width="24"
				height="24"
				viewBox="0 0 24 24"
				fill="none"
				stroke="currentColor"
				stroke-width="2"
			>
				<rect x="3" y="4" width="18" height="18" rx="2" ry="2"></rect>
				<line x1="16" y1="2" x2="16" y2="6"></line>
				<line x1="8" y1="2" x2="8" y2="6"></line>
				<line x1="3" y1="10" x2="21" y2="10"></line>
			</svg>
		</button>
	</div>

	{#if isOpen}
		<div
			class="date-picker absolute top-full left-0 z-50 mt-2 rounded-lg border-2 p-4 shadow-lg"
			style={pickerStyle}
		>
			{#if pickerView === 'day'}
				{#key `day-${viewDate.getFullYear()}-${viewDate.getMonth()}-${slideDirection}`}
					<div class={slideDirection === 'left' ? 'slide-left' : 'slide-right'}>
						<div class="mb-4 flex items-center justify-between">
							<button
								type="button"
								onclick={previousMonth}
								aria-label="Previous month"
								class="rounded bg-transparent p-2 text-text-primary"
							>
								<svg
									width="20"
									height="20"
									viewBox="0 0 20 20"
									fill="none"
									stroke="currentColor"
									stroke-width="2"
								>
									<polyline points="12 4 6 10 12 16"></polyline>
								</svg>
							</button>
							<button
								type="button"
								onclick={switchToMonthView}
								class="rounded bg-transparent px-3 py-1 font-bold
								text-text-primary
								transition-colors"
							>
								{monthNames[viewDate.getMonth()]}
								{viewDate.getFullYear()}
							</button>
							<button
								type="button"
								onclick={nextMonth}
								disabled={!canGoToNextMonth()}
								aria-label="Next month"
								class="rounded bg-transparent p-2 text-text-primary
								disabled:cursor-not-allowed
								disabled:opacity-40"
							>
								<svg
									width="20"
									height="20"
									viewBox="0 0 20 20"
									fill="none"
									stroke="currentColor"
									stroke-width="2"
								>
									<polyline points="8 4 14 10 8 16"></polyline>
								</svg>
							</button>
						</div>

						<div class="mb-2 grid grid-cols-7 gap-1">
							{#each ['Su', 'Mo', 'Tu', 'We', 'Th', 'Fr', 'Sa'] as day (day)}
								<div class="py-2 text-center text-sm font-medium text-text-secondary">
									{day}
								</div>
							{/each}
						</div>

						<div class="grid grid-cols-7 gap-1">
							{#each getCalendarDays(viewDate) as day, index (`day-cell-${viewDate.getFullYear()}-${viewDate.getMonth()}-${index}`)}
								{#if day === null}
									<div class="aspect-square"></div>
								{:else}
									{@const isDisabled = isAfterMax(
										new Date(viewDate.getFullYear(), viewDate.getMonth(), day)
									)}
									<button
										type="button"
										onclick={() => selectDay(day)}
										disabled={isDisabled}
										class="flex aspect-square items-center justify-center rounded transition-colors hover:opacity-80 disabled:cursor-not-allowed disabled:opacity-40"
										class:font-bold={isSelectedDay(day)}
										style={isSelectedDay(day)
											? 'background-color: var(--button-primary); color: var(--bg-primary);'
											: 'color: var(--text-primary);'}
									>
										{day}
									</button>
								{/if}
							{/each}
						</div>
					</div>
				{/key}
			{/if}

			{#if pickerView === 'month'}
				{#key `month-${viewDate.getFullYear()}-${slideDirection}`}
					<div class={slideDirection === 'left' ? 'slide-left' : 'slide-right'}>
						<div class="mb-4 flex items-center justify-between">
							<button
								type="button"
								onclick={previousYear}
								aria-label="Previous year"
								class="rounded bg-transparent p-2 text-text-primary"
							>
								<svg
									width="20"
									height="20"
									viewBox="0 0 20 20"
									fill="none"
									stroke="currentColor"
									stroke-width="2"
								>
									<polyline points="12 4 6 10 12 16"></polyline>
								</svg>
							</button>
							<button
								type="button"
								onclick={switchToYearView}
								class="rounded bg-transparent px-3 py-1 font-bold text-text-primary transition-colors"
							>
								{viewDate.getFullYear()}
							</button>
							<button
								type="button"
								onclick={nextYear}
								disabled={!canGoToNextYear()}
								aria-label="Next year"
								class="rounded bg-transparent p-2 text-text-primary disabled:cursor-not-allowed disabled:opacity-40"
							>
								<svg
									width="20"
									height="20"
									viewBox="0 0 20 20"
									fill="none"
									stroke="currentColor"
									stroke-width="2"
								>
									<polyline points="8 4 14 10 8 16"></polyline>
								</svg>
							</button>
						</div>

						<div class="grid grid-cols-3 gap-2">
							{#each monthNamesShort as month, index (index)}
								<button
									type="button"
									onclick={() => selectMonth(index)}
									disabled={isMonthDisabled(index)}
									class="rounded px-4 py-3 font-medium transition-colors disabled:cursor-not-allowed disabled:opacity-40"
									class:font-bold={viewDate.getMonth() === index}
									style={viewDate.getMonth() === index
										? 'background-color: var(--button-primary); color: var(--bg-primary);'
										: 'color: var(--text-primary); background-color: transparent;'}
								>
									{month}
								</button>
							{/each}
						</div>
					</div>
				{/key}
			{/if}

			{#if pickerView === 'year'}
				{#key `year-${Math.floor(viewDate.getFullYear() / 12)}-${slideDirection}`}
					<div class={slideDirection === 'left' ? 'slide-left' : 'slide-right'}>
						<div class="mb-4 flex items-center justify-between">
							<button
								type="button"
								onclick={previousYearRange}
								aria-label="Previous years"
								class="rounded bg-transparent p-2 text-text-primary"
							>
								<svg
									width="20"
									height="20"
									viewBox="0 0 20 20"
									fill="none"
									stroke="currentColor"
									stroke-width="2"
								>
									<polyline points="12 4 6 10 12 16"></polyline>
								</svg>
							</button>
							<div class="font-bold text-text-primary">
								{getYearRange()[0]} - {getYearRange()[11]}
							</div>
							<button
								type="button"
								onclick={nextYearRange}
								disabled={!canGoToNextYearRange()}
								aria-label="Next years"
								class="rounded bg-transparent p-2 text-text-primary disabled:cursor-not-allowed disabled:opacity-40"
							>
								<svg
									width="20"
									height="20"
									viewBox="0 0 20 20"
									fill="none"
									stroke="currentColor"
									stroke-width="2"
								>
									<polyline points="8 4 14 10 8 16"></polyline>
								</svg>
							</button>
						</div>

						<div class="grid grid-cols-3 gap-2">
							{#each getYearRange() as year (year)}
								<button
									type="button"
									onclick={() => selectYear(year)}
									disabled={isYearDisabled(year)}
									class="rounded px-4 py-3 font-medium transition-colors disabled:cursor-not-allowed disabled:opacity-40"
									class:font-bold={viewDate.getFullYear() === year}
									style={viewDate.getFullYear() === year
										? 'background-color: var(--button-primary); color: var(--bg-primary);'
										: 'color: var(--text-primary); background-color: transparent;'}
								>
									{year}
								</button>
							{/each}
						</div>
					</div>
				{/key}
			{/if}
		</div>
	{/if}
</div>

<style>
	@keyframes slideInFromRight {
		from {
			transform: translateX(100%);
			opacity: 0;
		}
		to {
			transform: translateX(0);
			opacity: 1;
		}
	}

	@keyframes slideInFromLeft {
		from {
			transform: translateX(-100%);
			opacity: 0;
		}
		to {
			transform: translateX(0);
			opacity: 1;
		}
	}

	.slide-left {
		animation: slideInFromLeft 0.3s ease-out;
	}

	.slide-right {
		animation: slideInFromRight 0.3s ease-out;
	}

	.date-picker button:not(:disabled) {
		cursor: pointer;
		transition:
			transform 0.12s ease,
			filter 0.12s ease;
	}

	.date-picker button:not(:disabled):hover {
		transform: translateY(-1px) scale(1.02);
		filter: brightness(0.96);
	}
</style>
