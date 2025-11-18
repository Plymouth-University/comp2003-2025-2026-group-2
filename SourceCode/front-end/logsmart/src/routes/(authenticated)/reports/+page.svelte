<script lang="ts">
	// Log types for selection
	let logTypes = [
		{ id: 'all', label: 'All', checked: true },
		{ id: 'type1', label: 'Log Type 1', checked: false },
		{ id: 'type2', label: 'Log Type 2', checked: false },
		{ id: 'type3', label: 'Log Type 3', checked: false },
		{ id: 'type4', label: 'Log Type 4', checked: false },
		{ id: 'type5', label: 'Log Type 5', checked: false }
	];

	// Get current date
	const today = new Date();
	const dd = String(today.getDate()).padStart(2, '0');
	const mm = String(today.getMonth() + 1).padStart(2, '0');
	const yyyy = today.getFullYear();
	const currentDateFormatted = `${dd}/${mm}/${yyyy}`;
	const currentDateISO = `${yyyy}-${mm}-${dd}`;

	// Date range - Store both formatted (DD/MM/YYYY) and ISO (YYYY-MM-DD) formats
	let dateFrom = $state(currentDateFormatted);
	let dateTo = $state(currentDateFormatted);
	let dateFromISO = $state(currentDateISO);
	let dateToISO = $state(currentDateISO);

	// Date picker visibility and view mode
	let showDateFromPicker = $state(false);
	let showDateToPicker = $state(false);
	let pickerView = $state<'day' | 'month' | 'year'>('day'); // day, month, or year view
	let slideDirection = $state<'left' | 'right'>('left');
	
	// Current month/year for calendar
	let calendarDate = $state(new Date());
	let activePickerIsFrom = $state(true);

	// Generated report state
	let reportGenerated = $state(false);

	// Convert DD/MM/YYYY to YYYY-MM-DD
	function formatToISO(dateStr: string): string {
		const parts = dateStr.split('/');
		if (parts.length === 3) {
			const [day, month, year] = parts;
			return `${year}-${month.padStart(2, '0')}-${day.padStart(2, '0')}`;
		}
		return '';
	}

	// Convert YYYY-MM-DD to DD/MM/YYYY
	function formatToDDMMYYYY(dateStr: string): string {
		const parts = dateStr.split('-');
		if (parts.length === 3) {
			const [year, month, day] = parts;
			return `${day}/${month}/${year}`;
		}
		return '';
	}

	// Update ISO date when text input changes
	function updateDateFromText(value: string, isFrom: boolean) {
		const iso = formatToISO(value);
		if (iso) {
			if (isFrom) {
				dateFromISO = iso;
			} else {
				dateToISO = iso;
			}
		}
	}

	// Toggle date picker
	function toggleDatePicker(isFrom: boolean) {
		activePickerIsFrom = isFrom;
		pickerView = 'day'; // Reset to day view when opening
		if (isFrom) {
			showDateFromPicker = !showDateFromPicker;
			showDateToPicker = false;
			// Set calendar to current date from value
			const parts = dateFrom.split('/');
			if (parts.length === 3) {
				calendarDate = new Date(parseInt(parts[2]), parseInt(parts[1]) - 1, parseInt(parts[0]));
			}
		} else {
			showDateToPicker = !showDateToPicker;
			showDateFromPicker = false;
			// Set calendar to current date to value
			const parts = dateTo.split('/');
			if (parts.length === 3) {
				calendarDate = new Date(parseInt(parts[2]), parseInt(parts[1]) - 1, parseInt(parts[0]));
			}
		}
	}

	// Switch between views
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

	// Get days in month
	function getDaysInMonth(date: Date): number {
		return new Date(date.getFullYear(), date.getMonth() + 1, 0).getDate();
	}

	// Get first day of month (0 = Sunday, 1 = Monday, etc.)
	function getFirstDayOfMonth(date: Date): number {
		return new Date(date.getFullYear(), date.getMonth(), 1).getDay();
	}

	// Generate calendar days
	function getCalendarDays(date: Date): (number | null)[] {
		const daysInMonth = getDaysInMonth(date);
		const firstDay = getFirstDayOfMonth(date);
		const days: (number | null)[] = [];
		
		// Add empty cells for days before the first day of the month
		for (let i = 0; i < firstDay; i++) {
			days.push(null);
		}
		
		// Add days of the month
		for (let i = 1; i <= daysInMonth; i++) {
			days.push(i);
		}
		
		return days;
	}

	// Navigate to previous month
	function previousMonth() {
		calendarDate = new Date(calendarDate.getFullYear(), calendarDate.getMonth() - 1, 1);
	}

	// Navigate to next month
	function nextMonth() {
		calendarDate = new Date(calendarDate.getFullYear(), calendarDate.getMonth() + 1, 1);
	}

	// Navigate to previous year
	function previousYear() {
		calendarDate = new Date(calendarDate.getFullYear() - 1, calendarDate.getMonth(), 1);
	}

	// Navigate to next year
	function nextYear() {
		calendarDate = new Date(calendarDate.getFullYear() + 1, calendarDate.getMonth(), 1);
	}

	// Navigate year range (for year view, show 12 years at a time)
	function previousYearRange() {
		calendarDate = new Date(calendarDate.getFullYear() - 12, calendarDate.getMonth(), 1);
	}

	function nextYearRange() {
		calendarDate = new Date(calendarDate.getFullYear() + 12, calendarDate.getMonth(), 1);
	}

	// Select a day
	function selectDay(day: number) {
		const selectedDate = new Date(calendarDate.getFullYear(), calendarDate.getMonth(), day);
		const dd = String(selectedDate.getDate()).padStart(2, '0');
		const mm = String(selectedDate.getMonth() + 1).padStart(2, '0');
		const yyyy = selectedDate.getFullYear();
		const formatted = `${dd}/${mm}/${yyyy}`;
		const iso = `${yyyy}-${mm}-${dd}`;
		
		if (activePickerIsFrom) {
			dateFrom = formatted;
			dateFromISO = iso;
			showDateFromPicker = false;
		} else {
			dateTo = formatted;
			dateToISO = iso;
			showDateToPicker = false;
		}
	}

	// Select a month
	function selectMonth(monthIndex: number) {
		calendarDate = new Date(calendarDate.getFullYear(), monthIndex, 1);
		switchToDayView();
	}

	// Select a year
	function selectYear(year: number) {
		calendarDate = new Date(year, calendarDate.getMonth(), 1);
		switchToMonthView();
	}

	// Check if a day is selected
	function isSelectedDay(day: number): boolean {
		const checkDate = new Date(calendarDate.getFullYear(), calendarDate.getMonth(), day);
		const formatted = `${String(day).padStart(2, '0')}/${String(calendarDate.getMonth() + 1).padStart(2, '0')}/${calendarDate.getFullYear()}`;
		
		if (activePickerIsFrom) {
			return dateFrom === formatted;
		} else {
			return dateTo === formatted;
		}
	}

	// Get year range for year view (12 years at a time)
	function getYearRange(): number[] {
		const currentYear = calendarDate.getFullYear();
		const startYear = Math.floor(currentYear / 12) * 12;
		const years = [];
		for (let i = 0; i < 12; i++) {
			years.push(startYear + i);
		}
		return years;
	}

	// Month names
	const monthNames = ['January', 'February', 'March', 'April', 'May', 'June',
		'July', 'August', 'September', 'October', 'November', 'December'];
	
	const monthNamesShort = ['Jan', 'Feb', 'Mar', 'Apr', 'May', 'Jun',
		'Jul', 'Aug', 'Sep', 'Oct', 'Nov', 'Dec'];

	function generateReport() {
		reportGenerated = true;
	}
</script>

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

<div class="min-h-screen" style="background-color: #F8F8F8;">
	<!-- Main Content -->
	<div class="mx-auto max-w-7xl px-6 py-8">
		<h1 class="text-4xl font-bold text-center mb-8" style="color: #000100;">Generate Report</h1>

		<div class="flex gap-8">
			<!-- Left Side - Form -->
			<div class="w-96">
				<!-- Date From -->
				<div class="mb-8">
					<label for="date-from" class="block text-lg font-bold mb-3" style="color: #000100;">Date From:</label>
					<div class="relative">
						<div class="flex items-center gap-2">
							<input 
								id="date-from"
								type="text" 
								bind:value={dateFrom}
								oninput={(e) => updateDateFromText(e.currentTarget.value, true)}
								placeholder="DD/MM/YYYY"
								class="px-4 py-2 border-2 flex-1"
								style="border-color: #000100;"
							/>
							<button 
								type="button"
								onclick={() => toggleDatePicker(true)}
								aria-label="Open calendar for start date" 
								class="border-2 p-2" 
								style="border-color: #000100;"
							>
								<svg width="24" height="24" viewBox="0 0 24 24" fill="none" stroke="#000100" stroke-width="2">
									<rect x="3" y="4" width="18" height="18" rx="2" ry="2"></rect>
									<line x1="16" y1="2" x2="16" y2="6"></line>
									<line x1="8" y1="2" x2="8" y2="6"></line>
									<line x1="3" y1="10" x2="21" y2="10"></line>
								</svg>
							</button>
						</div>
						
						<!-- Modern Date Picker -->
						{#if showDateFromPicker}
							<div class="absolute left-0 top-full mt-2 bg-white border-2 rounded-lg shadow-lg p-4 z-50" style="border-color: #000100; min-width: 320px; overflow: hidden;">
								<!-- Day View -->
								{#if pickerView === 'day'}
									<div class={slideDirection === 'left' ? 'slide-left' : 'slide-right'}>
										<!-- Month/Year Header -->
										<div class="flex items-center justify-between mb-4">
											<button type="button" onclick={previousMonth} aria-label="Previous month" class="p-2 hover:bg-gray-100 rounded" style="color: #000100;">
												<svg width="20" height="20" viewBox="0 0 20 20" fill="none" stroke="currentColor" stroke-width="2">
													<polyline points="12 4 6 10 12 16"></polyline>
												</svg>
											</button>
											<button type="button" onclick={switchToMonthView} class="font-bold hover:bg-gray-100 px-3 py-1 rounded transition-colors" style="color: #000100;">
												{monthNames[calendarDate.getMonth()]} {calendarDate.getFullYear()}
											</button>
											<button type="button" onclick={nextMonth} aria-label="Next month" class="p-2 hover:bg-gray-100 rounded" style="color: #000100;">
												<svg width="20" height="20" viewBox="0 0 20 20" fill="none" stroke="currentColor" stroke-width="2">
													<polyline points="8 4 14 10 8 16"></polyline>
												</svg>
											</button>
										</div>
										
										<!-- Day Labels -->
										<div class="grid grid-cols-7 gap-1 mb-2">
											{#each ['Su', 'Mo', 'Tu', 'We', 'Th', 'Fr', 'Sa'] as day}
												<div class="text-center text-sm font-medium py-2" style="color: #A1A6B4;">
													{day}
												</div>
											{/each}
										</div>
										
										<!-- Calendar Days -->
										<div class="grid grid-cols-7 gap-1">
											{#each getCalendarDays(calendarDate) as day}
												{#if day === null}
													<div class="aspect-square"></div>
												{:else}
													<button
														type="button"
														onclick={() => selectDay(day)}
														class="aspect-square flex items-center justify-center rounded hover:bg-gray-100 transition-colors"
														class:font-bold={isSelectedDay(day)}
														style={isSelectedDay(day) ? 'background-color: #94C5CC; color: white;' : 'color: #000100;'}
													>
														{day}
													</button>
												{/if}
											{/each}
										</div>
									</div>
								{/if}

								<!-- Month View -->
								{#if pickerView === 'month'}
									<div class={slideDirection === 'left' ? 'slide-left' : 'slide-right'}>
										<!-- Year Header -->
										<div class="flex items-center justify-between mb-4">
											<button type="button" onclick={previousYear} aria-label="Previous year" class="p-2 hover:bg-gray-100 rounded" style="color: #000100;">
												<svg width="20" height="20" viewBox="0 0 20 20" fill="none" stroke="currentColor" stroke-width="2">
													<polyline points="12 4 6 10 12 16"></polyline>
												</svg>
											</button>
											<button type="button" onclick={switchToYearView} class="font-bold hover:bg-gray-100 px-3 py-1 rounded transition-colors" style="color: #000100;">
												{calendarDate.getFullYear()}
											</button>
											<button type="button" onclick={nextYear} aria-label="Next year" class="p-2 hover:bg-gray-100 rounded" style="color: #000100;">
												<svg width="20" height="20" viewBox="0 0 20 20" fill="none" stroke="currentColor" stroke-width="2">
													<polyline points="8 4 14 10 8 16"></polyline>
												</svg>
											</button>
										</div>
										
										<!-- Month Grid -->
										<div class="grid grid-cols-3 gap-2">
											{#each monthNamesShort as month, index}
												<button
													type="button"
													onclick={() => selectMonth(index)}
													class="px-4 py-3 rounded hover:bg-gray-100 transition-colors font-medium"
													class:font-bold={calendarDate.getMonth() === index}
													style={calendarDate.getMonth() === index ? 'background-color: #94C5CC; color: white;' : 'color: #000100;'}
												>
													{month}
												</button>
											{/each}
										</div>
									</div>
								{/if}

								<!-- Year View -->
								{#if pickerView === 'year'}
									<div class={slideDirection === 'left' ? 'slide-left' : 'slide-right'}>
										<!-- Year Range Header -->
										<div class="flex items-center justify-between mb-4">
											<button type="button" onclick={previousYearRange} aria-label="Previous years" class="p-2 hover:bg-gray-100 rounded" style="color: #000100;">
												<svg width="20" height="20" viewBox="0 0 20 20" fill="none" stroke="currentColor" stroke-width="2">
													<polyline points="12 4 6 10 12 16"></polyline>
												</svg>
											</button>
											<div class="font-bold" style="color: #000100;">
												{getYearRange()[0]} - {getYearRange()[11]}
											</div>
											<button type="button" onclick={nextYearRange} aria-label="Next years" class="p-2 hover:bg-gray-100 rounded" style="color: #000100;">
												<svg width="20" height="20" viewBox="0 0 20 20" fill="none" stroke="currentColor" stroke-width="2">
													<polyline points="8 4 14 10 8 16"></polyline>
												</svg>
											</button>
										</div>
										
										<!-- Year Grid -->
										<div class="grid grid-cols-3 gap-2">
											{#each getYearRange() as year}
												<button
													type="button"
													onclick={() => selectYear(year)}
													class="px-4 py-3 rounded hover:bg-gray-100 transition-colors font-medium"
													class:font-bold={calendarDate.getFullYear() === year}
													style={calendarDate.getFullYear() === year ? 'background-color: #94C5CC; color: white;' : 'color: #000100;'}
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
				<div class="mb-8">
					<label for="date-to" class="block text-lg font-bold mb-3" style="color: #000100;">Date To:</label>
					<div class="relative">
						<div class="flex items-center gap-2">
							<input 
								id="date-to"
								type="text" 
								bind:value={dateTo}
								oninput={(e) => updateDateFromText(e.currentTarget.value, false)}
								placeholder="DD/MM/YYYY"
								class="px-4 py-2 border-2 flex-1"
								style="border-color: #000100;"
							/>
							<button 
								type="button"
								onclick={() => toggleDatePicker(false)}
								aria-label="Open calendar for end date" 
								class="border-2 p-2" 
								style="border-color: #000100;"
							>
								<svg width="24" height="24" viewBox="0 0 24 24" fill="none" stroke="#000100" stroke-width="2">
									<rect x="3" y="4" width="18" height="18" rx="2" ry="2"></rect>
									<line x1="16" y1="2" x2="16" y2="6"></line>
									<line x1="8" y1="2" x2="8" y2="6"></line>
									<line x1="3" y1="10" x2="21" y2="10"></line>
								</svg>
							</button>
						</div>
						
						<!-- Modern Date Picker -->
						{#if showDateToPicker}
							<div class="absolute left-0 top-full mt-2 bg-white border-2 rounded-lg shadow-lg p-4 z-50" style="border-color: #000100; min-width: 320px; overflow: hidden;">
								<!-- Day View -->
								{#if pickerView === 'day'}
									<div class={slideDirection === 'left' ? 'slide-left' : 'slide-right'}>
										<!-- Month/Year Header -->
										<div class="flex items-center justify-between mb-4">
											<button type="button" onclick={previousMonth} aria-label="Previous month" class="p-2 hover:bg-gray-100 rounded" style="color: #000100;">
												<svg width="20" height="20" viewBox="0 0 20 20" fill="none" stroke="currentColor" stroke-width="2">
													<polyline points="12 4 6 10 12 16"></polyline>
												</svg>
											</button>
											<button type="button" onclick={switchToMonthView} class="font-bold hover:bg-gray-100 px-3 py-1 rounded transition-colors" style="color: #000100;">
												{monthNames[calendarDate.getMonth()]} {calendarDate.getFullYear()}
											</button>
											<button type="button" onclick={nextMonth} aria-label="Next month" class="p-2 hover:bg-gray-100 rounded" style="color: #000100;">
												<svg width="20" height="20" viewBox="0 0 20 20" fill="none" stroke="currentColor" stroke-width="2">
													<polyline points="8 4 14 10 8 16"></polyline>
												</svg>
											</button>
										</div>
										
										<!-- Day Labels -->
										<div class="grid grid-cols-7 gap-1 mb-2">
											{#each ['Su', 'Mo', 'Tu', 'We', 'Th', 'Fr', 'Sa'] as day}
												<div class="text-center text-sm font-medium py-2" style="color: #A1A6B4;">
													{day}
												</div>
											{/each}
										</div>
										
										<!-- Calendar Days -->
										<div class="grid grid-cols-7 gap-1">
											{#each getCalendarDays(calendarDate) as day}
												{#if day === null}
													<div class="aspect-square"></div>
												{:else}
													<button
														type="button"
														onclick={() => selectDay(day)}
														class="aspect-square flex items-center justify-center rounded hover:bg-gray-100 transition-colors"
														class:font-bold={isSelectedDay(day)}
														style={isSelectedDay(day) ? 'background-color: #94C5CC; color: white;' : 'color: #000100;'}
													>
														{day}
													</button>
												{/if}
											{/each}
										</div>
									</div>
								{/if}

								<!-- Month View -->
								{#if pickerView === 'month'}
									<div class={slideDirection === 'left' ? 'slide-left' : 'slide-right'}>
										<!-- Year Header -->
										<div class="flex items-center justify-between mb-4">
											<button type="button" onclick={previousYear} aria-label="Previous year" class="p-2 hover:bg-gray-100 rounded" style="color: #000100;">
												<svg width="20" height="20" viewBox="0 0 20 20" fill="none" stroke="currentColor" stroke-width="2">
													<polyline points="12 4 6 10 12 16"></polyline>
												</svg>
											</button>
											<button type="button" onclick={switchToYearView} class="font-bold hover:bg-gray-100 px-3 py-1 rounded transition-colors" style="color: #000100;">
												{calendarDate.getFullYear()}
											</button>
											<button type="button" onclick={nextYear} aria-label="Next year" class="p-2 hover:bg-gray-100 rounded" style="color: #000100;">
												<svg width="20" height="20" viewBox="0 0 20 20" fill="none" stroke="currentColor" stroke-width="2">
													<polyline points="8 4 14 10 8 16"></polyline>
												</svg>
											</button>
										</div>
										
										<!-- Month Grid -->
										<div class="grid grid-cols-3 gap-2">
											{#each monthNamesShort as month, index}
												<button
													type="button"
													onclick={() => selectMonth(index)}
													class="px-4 py-3 rounded hover:bg-gray-100 transition-colors font-medium"
													class:font-bold={calendarDate.getMonth() === index}
													style={calendarDate.getMonth() === index ? 'background-color: #94C5CC; color: white;' : 'color: #000100;'}
												>
													{month}
												</button>
											{/each}
										</div>
									</div>
								{/if}

								<!-- Year View -->
								{#if pickerView === 'year'}
									<div class={slideDirection === 'left' ? 'slide-left' : 'slide-right'}>
										<!-- Year Range Header -->
										<div class="flex items-center justify-between mb-4">
											<button type="button" onclick={previousYearRange} aria-label="Previous years" class="p-2 hover:bg-gray-100 rounded" style="color: #000100;">
												<svg width="20" height="20" viewBox="0 0 20 20" fill="none" stroke="currentColor" stroke-width="2">
													<polyline points="12 4 6 10 12 16"></polyline>
												</svg>
											</button>
											<div class="font-bold" style="color: #000100;">
												{getYearRange()[0]} - {getYearRange()[11]}
											</div>
											<button type="button" onclick={nextYearRange} aria-label="Next years" class="p-2 hover:bg-gray-100 rounded" style="color: #000100;">
												<svg width="20" height="20" viewBox="0 0 20 20" fill="none" stroke="currentColor" stroke-width="2">
													<polyline points="8 4 14 10 8 16"></polyline>
												</svg>
											</button>
										</div>
										
										<!-- Year Grid -->
										<div class="grid grid-cols-3 gap-2">
											{#each getYearRange() as year}
												<button
													type="button"
													onclick={() => selectYear(year)}
													class="px-4 py-3 rounded hover:bg-gray-100 transition-colors font-medium"
													class:font-bold={calendarDate.getFullYear() === year}
													style={calendarDate.getFullYear() === year ? 'background-color: #94C5CC; color: white;' : 'color: #000100;'}
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

				<!-- Log Types -->
				<div class="mb-8">
					<fieldset>
						<legend class="block text-lg font-bold mb-3" style="color: #000100;">Log Types:</legend>
						<div class="space-y-2">
							{#each logTypes as logType}
								<label class="flex items-center gap-3 cursor-pointer">
									<input 
										type="checkbox" 
										bind:checked={logType.checked}
										class="w-5 h-5 border-2 cursor-pointer"
										style="border-color: #000100;"
									/>
									<span style="color: #000100;">{logType.label}</span>
								</label>
							{/each}
						</div>
					</fieldset>
				</div>

				<!-- Generate Button -->
				<div class="flex justify-center">
					<button 
						onclick={generateReport}
						class="px-8 py-2 border-2 font-medium hover:opacity-80 flex items-center gap-2"
						style="border-color: #000100; background-color: #A1A6B4; color: #000100;"
					>
						Generate
						<svg width="16" height="16" viewBox="0 0 16 16" fill="none" stroke="#000100" stroke-width="2">
							<polyline points="6 3 11 8 6 13"></polyline>
						</svg>
					</button>
				</div>
			</div>

			<!-- Right Side - Report Preview -->
			<div class="flex-1">
				<!-- Download Buttons -->
				<div class="flex justify-end gap-4 mb-4">
					<button class="px-4 py-2 border-2 font-medium hover:opacity-80" style="border-color: #000100; color: #000100; background-color: white;">
						Download PDF
					</button>
					<button class="px-4 py-2 border-2 font-medium hover:opacity-80" style="border-color: #000100; color: #000100; background-color: white;">
						Download DOCX
					</button>
					<button class="px-4 py-2 border-2 font-medium hover:opacity-80" style="border-color: #000100; color: #000100; background-color: white;">
						Download RTF
					</button>
				</div>

				<!-- Report Preview Area -->
				<div class="border-2 bg-white p-8 min-h-[600px]" style="border-color: #000100;">
					{#if reportGenerated}
						<div class="flex items-start gap-3">
							<svg width="32" height="32" viewBox="0 0 32 32" fill="none" stroke="#000100" stroke-width="2">
								<polyline points="9 17 14 22 23 10"></polyline>
								<rect x="4" y="4" width="24" height="24" rx="2" ry="2"></rect>
							</svg>
							<div>
								<p style="color: #000100;">Report generated successfully</p>
							</div>
						</div>
					{:else}
						<div class="flex items-start gap-3">
							<svg width="32" height="32" viewBox="0 0 32 32" fill="none" stroke="#A1A6B4" stroke-width="2">
								<polyline points="9 17 14 22 23 10"></polyline>
								<rect x="4" y="4" width="24" height="24" rx="2" ry="2"></rect>
							</svg>
							<p style="color: #A1A6B4;">Generate a report to preview</p>
						</div>
					{/if}
				</div>
			</div>
		</div>
	</div>
</div>
