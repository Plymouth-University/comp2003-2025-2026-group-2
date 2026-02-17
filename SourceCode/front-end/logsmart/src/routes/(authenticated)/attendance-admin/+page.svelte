<script lang="ts">
	import type { PageData } from './$types';
	import { goto } from '$app/navigation';
	import { page } from '$app/state';

	let { data } = $props<{ data: PageData }>();

	const clockEvents = $derived(data?.clockEvents ?? []);

	// --- Date picker state (matches reports page) ---
	const today = new Date();
	const dd = String(today.getDate()).padStart(2, '0');
	const mm = String(today.getMonth() + 1).padStart(2, '0');
	const yyyy = today.getFullYear();
	const currentDateFormatted = `${dd}/${mm}/${yyyy}`;

	// Initialise from URL params (convert ISO → DD/MM/YYYY)
	function isoToDisplay(iso: string | null): string {
		if (!iso) return '';
		const d = new Date(iso);
		if (isNaN(d.getTime())) return '';
		return `${String(d.getDate()).padStart(2, '0')}/${String(d.getMonth() + 1).padStart(2, '0')}/${d.getFullYear()}`;
	}
	function displayToISO(display: string): string {
		const parts = display.split('/');
		if (parts.length === 3) {
			const [day, month, year] = parts;
			return `${year}-${month.padStart(2, '0')}-${day.padStart(2, '0')}`;
		}
		return '';
	}

	let dateFrom = $state(isoToDisplay(page.url.searchParams.get('from')) || '');
	let dateTo = $state(isoToDisplay(page.url.searchParams.get('to')) || '');

	let showDateFromPicker = $state(false);
	let showDateToPicker = $state(false);
	let pickerView = $state<'day' | 'month' | 'year'>('day');
	let slideDirection = $state<'left' | 'right'>('left');
	let calendarDate = $state(new Date());
	let activePickerIsFrom = $state(true);

	// Search / filter
	let searchQuery = $state('');

	const filteredEvents = $derived(() => {
		if (!searchQuery.trim()) return clockEvents;
		const q = searchQuery.toLowerCase();
		return clockEvents.filter(
			(e: any) =>
				e.first_name.toLowerCase().includes(q) ||
				e.last_name.toLowerCase().includes(q) ||
				e.email.toLowerCase().includes(q)
		);
	});

	// --- Date picker helpers ---
	function updateDateFromText(value: string, isFrom: boolean) {
		// no-op for now, value is bound directly
	}

	function toggleDatePicker(isFrom: boolean) {
		activePickerIsFrom = isFrom;
		pickerView = 'day';
		if (isFrom) {
			showDateFromPicker = !showDateFromPicker;
			showDateToPicker = false;
			const parts = dateFrom.split('/');
			if (parts.length === 3) {
				calendarDate = new Date(parseInt(parts[2]), parseInt(parts[1]) - 1, parseInt(parts[0]));
			}
		} else {
			showDateToPicker = !showDateToPicker;
			showDateFromPicker = false;
			const parts = dateTo.split('/');
			if (parts.length === 3) {
				calendarDate = new Date(parseInt(parts[2]), parseInt(parts[1]) - 1, parseInt(parts[0]));
			}
		}
	}

	function switchToMonthView() {
		slideDirection = 'right';
		pickerView = 'month';
	}
	function switchToYearView() {
		slideDirection = 'right';
		pickerView = 'year';
	}
	function switchToDayView() {
		slideDirection = 'left';
		pickerView = 'day';
	}

	function getDaysInMonth(date: Date): number {
		return new Date(date.getFullYear(), date.getMonth() + 1, 0).getDate();
	}
	function getFirstDayOfMonth(date: Date): number {
		return new Date(date.getFullYear(), date.getMonth(), 1).getDay();
	}
	function getCalendarDays(date: Date): (number | null)[] {
		const daysInMonth = getDaysInMonth(date);
		const firstDay = getFirstDayOfMonth(date);
		const days: (number | null)[] = [];
		for (let i = 0; i < firstDay; i++) days.push(null);
		for (let i = 1; i <= daysInMonth; i++) days.push(i);
		return days;
	}

	function previousMonth() {
		calendarDate = new Date(calendarDate.getFullYear(), calendarDate.getMonth() - 1, 1);
	}
	function nextMonth() {
		calendarDate = new Date(calendarDate.getFullYear(), calendarDate.getMonth() + 1, 1);
	}
	function previousYear() {
		calendarDate = new Date(calendarDate.getFullYear() - 1, calendarDate.getMonth(), 1);
	}
	function nextYear() {
		calendarDate = new Date(calendarDate.getFullYear() + 1, calendarDate.getMonth(), 1);
	}
	function previousYearRange() {
		calendarDate = new Date(calendarDate.getFullYear() - 12, calendarDate.getMonth(), 1);
	}
	function nextYearRange() {
		calendarDate = new Date(calendarDate.getFullYear() + 12, calendarDate.getMonth(), 1);
	}

	function selectDay(day: number) {
		const selectedDate = new Date(calendarDate.getFullYear(), calendarDate.getMonth(), day);
		const d = String(selectedDate.getDate()).padStart(2, '0');
		const m = String(selectedDate.getMonth() + 1).padStart(2, '0');
		const y = selectedDate.getFullYear();
		const formatted = `${d}/${m}/${y}`;
		if (activePickerIsFrom) {
			dateFrom = formatted;
			showDateFromPicker = false;
		} else {
			dateTo = formatted;
			showDateToPicker = false;
		}
	}
	function selectMonth(monthIndex: number) {
		calendarDate = new Date(calendarDate.getFullYear(), monthIndex, 1);
		switchToDayView();
	}
	function selectYear(year: number) {
		calendarDate = new Date(year, calendarDate.getMonth(), 1);
		switchToMonthView();
	}

	function isSelectedDay(day: number): boolean {
		const formatted = `${String(day).padStart(2, '0')}/${String(calendarDate.getMonth() + 1).padStart(2, '0')}/${calendarDate.getFullYear()}`;
		return activePickerIsFrom ? dateFrom === formatted : dateTo === formatted;
	}

	function getYearRange(): number[] {
		const currentYear = calendarDate.getFullYear();
		const startYear = Math.floor(currentYear / 12) * 12;
		const years = [];
		for (let i = 0; i < 12; i++) years.push(startYear + i);
		return years;
	}

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

	// --- Apply / Clear ---
	function applyDateFilter() {
		const params = new URLSearchParams();
		const fromISO = displayToISO(dateFrom);
		const toISO = displayToISO(dateTo);
		if (fromISO) params.set('from', new Date(fromISO).toISOString());
		if (toISO) {
			const end = new Date(toISO);
			end.setHours(23, 59, 59, 999);
			params.set('to', end.toISOString());
		}
		const qs = params.toString();
		goto(`/attendance-admin${qs ? '?' + qs : ''}`, { invalidateAll: true });
	}

	function clearFilters() {
		dateFrom = '';
		dateTo = '';
		searchQuery = '';
		goto('/attendance-admin', { invalidateAll: true });
	}

	function formatDateTime(iso: string) {
		if (!iso) return '—';
		const d = new Date(iso);
		return d.toLocaleString('en-GB', {
			day: '2-digit',
			month: 'short',
			year: 'numeric',
			hour: '2-digit',
			minute: '2-digit'
		});
	}

	function formatDuration(clockIn: string, clockOut: string | null) {
		if (!clockOut) return 'In progress';
		const ms = new Date(clockOut).getTime() - new Date(clockIn).getTime();
		const hours = Math.floor(ms / 3600000);
		const mins = Math.floor((ms % 3600000) / 60000);
		return `${hours}h ${mins}m`;
	}

	function printToPDF() {
		const events = filteredEvents();
		const companyName = data?.user?.company_name ?? 'Company';
		const dateRange =
			dateFrom || dateTo ? `${dateFrom || 'Start'} to ${dateTo || 'Present'}` : 'All time';

		let html = `<!DOCTYPE html><html><head><title>Attendance Report - ${companyName}</title>
<style>
	body { font-family: Arial, sans-serif; margin: 20px; color: #333; }
	h1 { color: #3D7A82; margin-bottom: 4px; }
	.meta { color: #666; margin-bottom: 16px; font-size: 14px; }
	table { width: 100%; border-collapse: collapse; font-size: 13px; }
	th { background-color: #3D7A82; color: white; text-align: left; padding: 8px 12px; }
	td { padding: 8px 12px; border-bottom: 1px solid #ddd; }
	tr:nth-child(even) { background-color: #f8f9fa; }
	.status-in { color: #16a34a; font-weight: bold; }
	.status-out { color: #6b7280; }
	.footer { margin-top: 20px; font-size: 11px; color: #999; }
	@media print { body { margin: 0; } }
</style></head><body>
<h1>Attendance Report</h1>
<div class="meta">${companyName} &mdash; ${dateRange} &mdash; Generated ${new Date().toLocaleString('en-GB')}</div>
<table>
<thead><tr><th>Employee</th><th>Email</th><th>Clock In</th><th>Clock Out</th><th>Duration</th><th>Status</th></tr></thead>
<tbody>`;

		for (const e of events) {
			const statusClass = e.status === 'in' ? 'status-in' : 'status-out';
			html += `<tr>
	<td>${e.first_name} ${e.last_name}</td>
	<td>${e.email}</td>
	<td>${formatDateTime(e.clock_in)}</td>
	<td>${e.clock_out ? formatDateTime(e.clock_out) : '—'}</td>
	<td>${formatDuration(e.clock_in, e.clock_out)}</td>
	<td class="${statusClass}">${e.status === 'in' ? 'Clocked In' : 'Clocked Out'}</td>
</tr>`;
		}

		html += `</tbody></table>
<div class="footer">LogSmart Attendance Report &mdash; ${events.length} record(s)</div>
</body></html>`;

		const w = window.open('', '_blank');
		if (w) {
			w.document.write(html);
			w.document.close();
			setTimeout(() => {
				w.print();
			}, 400);
		}
	}
</script>

<svelte:head>
	<title>Attendance</title>
</svelte:head>

<div class="h-full w-full" style="background-color: var(--bg-secondary);">
	<div class="mx-auto max-w-7xl px-6 py-8">
		<!-- Header -->
		<div class="mb-6 flex flex-wrap items-center justify-between gap-4">
			<h1 class="text-3xl font-bold" style="color: var(--text-primary);">Attendance Overview</h1>
			<button
				onclick={printToPDF}
				class="cursor-pointer rounded border-2 px-5 py-2 font-medium transition-opacity hover:opacity-80"
				style="border-color: #3D7A82; color: white; background-color: #3D7A82;"
			>
				Export PDF
			</button>
		</div>

		<!-- Filters -->
		<div
			class="mb-6 flex flex-wrap items-end gap-4 rounded border-2 p-4"
			style="border-color: var(--border-primary); background-color: var(--bg-primary);"
		>
			<!-- Date From -->
			<div class="flex flex-col gap-1">
				<label for="filter-from" class="text-xs font-medium" style="color: var(--text-secondary);"
					>From</label
				>
				<div class="relative">
					<div class="flex items-center gap-2">
						<input
							id="filter-from"
							type="text"
							bind:value={dateFrom}
							oninput={(e) => updateDateFromText(e.currentTarget.value, true)}
							placeholder="DD/MM/YYYY"
							class="border-2 px-3 py-2 text-sm"
							style="border-color: var(--border-primary); background-color: var(--bg-secondary); color: var(--text-primary); width: 140px;"
						/>
						<button
							type="button"
							onclick={() => toggleDatePicker(true)}
							aria-label="Open calendar for start date"
							class="border-2 p-2"
							style="border-color: var(--border-primary); background-color: var(--bg-secondary); color: var(--text-primary);"
						>
							<svg
								width="20"
								height="20"
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

					{#if showDateFromPicker}
						<div
							class="absolute top-full left-0 z-50 mt-2 rounded-lg border-2 p-4 shadow-lg"
							style="border-color: var(--border-primary); background-color: var(--bg-primary); min-width: 280px; overflow: hidden;"
						>
							{#if pickerView === 'day'}
								<div class={slideDirection === 'left' ? 'slide-left' : 'slide-right'}>
									<div class="mb-4 flex items-center justify-between">
										<button
											type="button"
											onclick={previousMonth}
											aria-label="Previous month"
											class="rounded p-2 hover:bg-gray-100"
											style="color: #000100;"
										>
											<svg
												width="20"
												height="20"
												viewBox="0 0 20 20"
												fill="none"
												stroke="currentColor"
												stroke-width="2"><polyline points="12 4 6 10 12 16"></polyline></svg
											>
										</button>
										<button
											type="button"
											onclick={switchToMonthView}
											class="rounded px-3 py-1 font-bold transition-colors hover:bg-gray-100"
											style="color: #000100;"
										>
											{monthNames[calendarDate.getMonth()]}
											{calendarDate.getFullYear()}
										</button>
										<button
											type="button"
											onclick={nextMonth}
											aria-label="Next month"
											class="rounded p-2 hover:bg-gray-100"
											style="color: #000100;"
										>
											<svg
												width="20"
												height="20"
												viewBox="0 0 20 20"
												fill="none"
												stroke="currentColor"
												stroke-width="2"><polyline points="8 4 14 10 8 16"></polyline></svg
											>
										</button>
									</div>
									<div class="mb-2 grid grid-cols-7 gap-1">
										{#each ['Su', 'Mo', 'Tu', 'We', 'Th', 'Fr', 'Sa'] as day}
											<div class="py-2 text-center text-sm font-medium" style="color: #A1A6B4;">
												{day}
											</div>
										{/each}
									</div>
									<div class="grid grid-cols-7 gap-1">
										{#each getCalendarDays(calendarDate) as day}
											{#if day === null}
												<div class="aspect-square"></div>
											{:else}
												<button
													type="button"
													onclick={() => selectDay(day)}
													class="flex aspect-square items-center justify-center rounded transition-colors hover:opacity-80"
													class:font-bold={isSelectedDay(day)}
													style={isSelectedDay(day)
														? 'background-color: #3D7A82; color: white;'
														: 'color: var(--text-primary);'}
												>
													{day}
												</button>
											{/if}
										{/each}
									</div>
								</div>
							{/if}
							{#if pickerView === 'month'}
								<div class={slideDirection === 'left' ? 'slide-left' : 'slide-right'}>
									<div class="mb-4 flex items-center justify-between">
										<button
											type="button"
											onclick={previousYear}
											aria-label="Previous year"
											class="rounded p-2"
											style="color: var(--text-primary); background-color: transparent;"
										>
											<svg
												width="20"
												height="20"
												viewBox="0 0 20 20"
												fill="none"
												stroke="currentColor"
												stroke-width="2"><polyline points="12 4 6 10 12 16"></polyline></svg
											>
										</button>
										<button
											type="button"
											onclick={switchToYearView}
											class="rounded px-3 py-1 font-bold transition-colors"
											style="color: var(--text-primary); background-color: transparent;"
										>
											{calendarDate.getFullYear()}
										</button>
										<button
											type="button"
											onclick={nextYear}
											aria-label="Next year"
											class="rounded p-2"
											style="color: var(--text-primary); background-color: transparent;"
										>
											<svg
												width="20"
												height="20"
												viewBox="0 0 20 20"
												fill="none"
												stroke="currentColor"
												stroke-width="2"><polyline points="8 4 14 10 8 16"></polyline></svg
											>
										</button>
									</div>
									<div class="grid grid-cols-3 gap-2">
										{#each monthNamesShort as month, index}
											<button
												type="button"
												onclick={() => selectMonth(index)}
												class="rounded px-4 py-3 font-medium transition-colors"
												class:font-bold={calendarDate.getMonth() === index}
												style={calendarDate.getMonth() === index
													? 'background-color: #3D7A82; color: white;'
													: 'color: var(--text-primary); background-color: transparent;'}
											>
												{month}
											</button>
										{/each}
									</div>
								</div>
							{/if}
							{#if pickerView === 'year'}
								<div class={slideDirection === 'left' ? 'slide-left' : 'slide-right'}>
									<div class="mb-4 flex items-center justify-between">
										<button
											type="button"
											onclick={previousYearRange}
											aria-label="Previous years"
											class="rounded p-2"
											style="color: var(--text-primary); background-color: transparent;"
										>
											<svg
												width="20"
												height="20"
												viewBox="0 0 20 20"
												fill="none"
												stroke="currentColor"
												stroke-width="2"><polyline points="12 4 6 10 12 16"></polyline></svg
											>
										</button>
										<div class="font-bold" style="color: var(--text-primary);">
											{getYearRange()[0]} - {getYearRange()[11]}
										</div>
										<button
											type="button"
											onclick={nextYearRange}
											aria-label="Next years"
											class="rounded p-2"
											style="color: var(--text-primary); background-color: transparent;"
										>
											<svg
												width="20"
												height="20"
												viewBox="0 0 20 20"
												fill="none"
												stroke="currentColor"
												stroke-width="2"><polyline points="8 4 14 10 8 16"></polyline></svg
											>
										</button>
									</div>
									<div class="grid grid-cols-3 gap-2">
										{#each getYearRange() as year}
											<button
												type="button"
												onclick={() => selectYear(year)}
												class="rounded px-4 py-3 font-medium transition-colors"
												class:font-bold={calendarDate.getFullYear() === year}
												style={calendarDate.getFullYear() === year
													? 'background-color: #3D7A82; color: white;'
													: 'color: var(--text-primary); background-color: transparent;'}
											>
												{year}
											</button>
										{/each}
									</div>
								</div>
							{/if}
						</div>
					{/if}
				</div>
			</div>

			<!-- Date To -->
			<div class="flex flex-col gap-1">
				<label for="filter-to" class="text-xs font-medium" style="color: var(--text-secondary);"
					>To</label
				>
				<div class="relative">
					<div class="flex items-center gap-2">
						<input
							id="filter-to"
							type="text"
							bind:value={dateTo}
							oninput={(e) => updateDateFromText(e.currentTarget.value, false)}
							placeholder="DD/MM/YYYY"
							class="border-2 px-3 py-2 text-sm"
							style="border-color: var(--border-primary); background-color: var(--bg-secondary); color: var(--text-primary); width: 140px;"
						/>
						<button
							type="button"
							onclick={() => toggleDatePicker(false)}
							aria-label="Open calendar for end date"
							class="border-2 p-2"
							style="border-color: var(--border-primary); background-color: var(--bg-secondary); color: var(--text-primary);"
						>
							<svg
								width="20"
								height="20"
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

					{#if showDateToPicker}
						<div
							class="absolute top-full left-0 z-50 mt-2 rounded-lg border-2 p-4 shadow-lg"
							style="border-color: var(--border-primary); background-color: var(--bg-primary); min-width: 280px; overflow: hidden;"
						>
							{#if pickerView === 'day'}
								<div class={slideDirection === 'left' ? 'slide-left' : 'slide-right'}>
									<div class="mb-4 flex items-center justify-between">
										<button
											type="button"
											onclick={previousMonth}
											aria-label="Previous month"
											class="rounded p-2 hover:bg-gray-100"
											style="color: #000100;"
										>
											<svg
												width="20"
												height="20"
												viewBox="0 0 20 20"
												fill="none"
												stroke="currentColor"
												stroke-width="2"><polyline points="12 4 6 10 12 16"></polyline></svg
											>
										</button>
										<button
											type="button"
											onclick={switchToMonthView}
											class="rounded px-3 py-1 font-bold transition-colors hover:bg-gray-100"
											style="color: #000100;"
										>
											{monthNames[calendarDate.getMonth()]}
											{calendarDate.getFullYear()}
										</button>
										<button
											type="button"
											onclick={nextMonth}
											aria-label="Next month"
											class="rounded p-2 hover:bg-gray-100"
											style="color: #000100;"
										>
											<svg
												width="20"
												height="20"
												viewBox="0 0 20 20"
												fill="none"
												stroke="currentColor"
												stroke-width="2"><polyline points="8 4 14 10 8 16"></polyline></svg
											>
										</button>
									</div>
									<div class="mb-2 grid grid-cols-7 gap-1">
										{#each ['Su', 'Mo', 'Tu', 'We', 'Th', 'Fr', 'Sa'] as day}
											<div class="py-2 text-center text-sm font-medium" style="color: #A1A6B4;">
												{day}
											</div>
										{/each}
									</div>
									<div class="grid grid-cols-7 gap-1">
										{#each getCalendarDays(calendarDate) as day}
											{#if day === null}
												<div class="aspect-square"></div>
											{:else}
												<button
													type="button"
													onclick={() => selectDay(day)}
													class="flex aspect-square items-center justify-center rounded transition-colors hover:opacity-80"
													class:font-bold={isSelectedDay(day)}
													style={isSelectedDay(day)
														? 'background-color: #3D7A82; color: white;'
														: 'color: var(--text-primary);'}
												>
													{day}
												</button>
											{/if}
										{/each}
									</div>
								</div>
							{/if}
							{#if pickerView === 'month'}
								<div class={slideDirection === 'left' ? 'slide-left' : 'slide-right'}>
									<div class="mb-4 flex items-center justify-between">
										<button
											type="button"
											onclick={previousYear}
											aria-label="Previous year"
											class="rounded p-2"
											style="color: var(--text-primary); background-color: transparent;"
										>
											<svg
												width="20"
												height="20"
												viewBox="0 0 20 20"
												fill="none"
												stroke="currentColor"
												stroke-width="2"><polyline points="12 4 6 10 12 16"></polyline></svg
											>
										</button>
										<button
											type="button"
											onclick={switchToYearView}
											class="rounded px-3 py-1 font-bold transition-colors"
											style="color: var(--text-primary); background-color: transparent;"
										>
											{calendarDate.getFullYear()}
										</button>
										<button
											type="button"
											onclick={nextYear}
											aria-label="Next year"
											class="rounded p-2"
											style="color: var(--text-primary); background-color: transparent;"
										>
											<svg
												width="20"
												height="20"
												viewBox="0 0 20 20"
												fill="none"
												stroke="currentColor"
												stroke-width="2"><polyline points="8 4 14 10 8 16"></polyline></svg
											>
										</button>
									</div>
									<div class="grid grid-cols-3 gap-2">
										{#each monthNamesShort as month, index}
											<button
												type="button"
												onclick={() => selectMonth(index)}
												class="rounded px-4 py-3 font-medium transition-colors"
												class:font-bold={calendarDate.getMonth() === index}
												style={calendarDate.getMonth() === index
													? 'background-color: #3D7A82; color: white;'
													: 'color: var(--text-primary); background-color: transparent;'}
											>
												{month}
											</button>
										{/each}
									</div>
								</div>
							{/if}
							{#if pickerView === 'year'}
								<div class={slideDirection === 'left' ? 'slide-left' : 'slide-right'}>
									<div class="mb-4 flex items-center justify-between">
										<button
											type="button"
											onclick={previousYearRange}
											aria-label="Previous years"
											class="rounded p-2"
											style="color: var(--text-primary); background-color: transparent;"
										>
											<svg
												width="20"
												height="20"
												viewBox="0 0 20 20"
												fill="none"
												stroke="currentColor"
												stroke-width="2"><polyline points="12 4 6 10 12 16"></polyline></svg
											>
										</button>
										<div class="font-bold" style="color: var(--text-primary);">
											{getYearRange()[0]} - {getYearRange()[11]}
										</div>
										<button
											type="button"
											onclick={nextYearRange}
											aria-label="Next years"
											class="rounded p-2"
											style="color: var(--text-primary); background-color: transparent;"
										>
											<svg
												width="20"
												height="20"
												viewBox="0 0 20 20"
												fill="none"
												stroke="currentColor"
												stroke-width="2"><polyline points="8 4 14 10 8 16"></polyline></svg
											>
										</button>
									</div>
									<div class="grid grid-cols-3 gap-2">
										{#each getYearRange() as year}
											<button
												type="button"
												onclick={() => selectYear(year)}
												class="rounded px-4 py-3 font-medium transition-colors"
												class:font-bold={calendarDate.getFullYear() === year}
												style={calendarDate.getFullYear() === year
													? 'background-color: #3D7A82; color: white;'
													: 'color: var(--text-primary); background-color: transparent;'}
											>
												{year}
											</button>
										{/each}
									</div>
								</div>
							{/if}
						</div>
					{/if}
				</div>
			</div>

			<button
				onclick={applyDateFilter}
				class="cursor-pointer rounded border-2 px-4 py-2 text-sm font-medium transition-opacity hover:opacity-80"
				style="border-color: #3D7A82; color: white; background-color: #3D7A82;"
			>
				Apply
			</button>
			<button
				onclick={clearFilters}
				class="cursor-pointer rounded border-2 px-4 py-2 text-sm font-medium transition-opacity hover:opacity-80"
				style="border-color: var(--border-primary); color: var(--text-primary); background-color: var(--bg-primary);"
			>
				Clear
			</button>

			<div class="ml-auto flex flex-col gap-1">
				<label for="filter-search" class="text-xs font-medium" style="color: var(--text-secondary);"
					>Search</label
				>
				<input
					id="filter-search"
					type="text"
					bind:value={searchQuery}
					placeholder="Filter by name or email..."
					class="rounded border px-3 py-2 text-sm"
					style="border-color: var(--border-primary); background-color: var(--bg-secondary); color: var(--text-primary); min-width: 220px;"
				/>
			</div>
		</div>

		<!-- Summary -->
		<div class="mb-4 text-sm" style="color: var(--text-secondary);">
			{filteredEvents().length} record{filteredEvents().length !== 1 ? 's' : ''}
			{#if searchQuery.trim()}
				&mdash; filtered by &ldquo;{searchQuery}&rdquo;
			{/if}
		</div>

		<!-- Table -->
		<div
			class="overflow-x-auto rounded border-2"
			style="border-color: var(--border-primary); background-color: var(--bg-primary);"
		>
			<table class="w-full text-left text-sm">
				<thead>
					<tr style="background-color: #3D7A82; color: white;">
						<th class="px-4 py-3 font-medium">Employee</th>
						<th class="px-4 py-3 font-medium">Email</th>
						<th class="px-4 py-3 font-medium">Clock In</th>
						<th class="px-4 py-3 font-medium">Clock Out</th>
						<th class="px-4 py-3 font-medium">Duration</th>
						<th class="px-4 py-3 font-medium">Status</th>
					</tr>
				</thead>
				<tbody>
					{#if filteredEvents().length === 0}
						<tr>
							<td colspan="6" class="px-4 py-8 text-center" style="color: var(--text-secondary);">
								No attendance records found
							</td>
						</tr>
					{:else}
						{#each filteredEvents() as event (event.id)}
							<tr
								class="border-b transition-colors hover:opacity-90"
								style="border-color: var(--border-secondary);"
							>
								<td class="px-4 py-3" style="color: var(--text-primary);">
									{event.first_name}
									{event.last_name}
								</td>
								<td class="px-4 py-3" style="color: var(--text-secondary);">
									{event.email}
								</td>
								<td class="px-4 py-3" style="color: var(--text-primary);">
									{formatDateTime(event.clock_in)}
								</td>
								<td class="px-4 py-3" style="color: var(--text-primary);">
									{event.clock_out ? formatDateTime(event.clock_out) : '—'}
								</td>
								<td class="px-4 py-3" style="color: var(--text-primary);">
									{formatDuration(event.clock_in, event.clock_out)}
								</td>
								<td class="px-4 py-3">
									{#if event.status === 'in'}
										<span
											class="inline-block rounded-full px-2 py-0.5 text-xs font-semibold"
											style="background-color: #dcfce7; color: #16a34a;"
										>
											Clocked In
										</span>
									{:else}
										<span
											class="inline-block rounded-full px-2 py-0.5 text-xs font-semibold"
											style="background-color: #f3f4f6; color: #6b7280;"
										>
											Clocked Out
										</span>
									{/if}
								</td>
							</tr>
						{/each}
					{/if}
				</tbody>
			</table>
		</div>
	</div>
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
</style>
