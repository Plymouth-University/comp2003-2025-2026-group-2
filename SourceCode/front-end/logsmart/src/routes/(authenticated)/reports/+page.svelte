<script lang="ts">
	import { api } from '$lib/api';
	import type { components } from '$lib/api-types';

	type LogEntry = components['schemas']['LogEntryResponse'];

	let logTypes = $state([
		{ id: 'all', label: 'All', checked: true },
		{ id: 'type1', label: 'Text Logs', checked: true },
		{ id: 'type2', label: 'Checkbox Logs', checked: true },
		{ id: 'type3', label: 'Temperature Logs', checked: true },
		{ id: 'type4', label: 'Dropdown Logs', checked: true }
	]);

	const today = new Date();
	const dd = String(today.getDate()).padStart(2, '0');
	const mm = String(today.getMonth() + 1).padStart(2, '0');
	const yyyy = today.getFullYear();
	const currentDateFormatted = `${dd}/${mm}/${yyyy}`;
	const currentDateISO = `${yyyy}-${mm}-${dd}`;

	let dateFrom = $state(currentDateFormatted);
	let dateTo = $state(currentDateFormatted);
	let dateFromISO = $state(currentDateISO);
	let dateToISO = $state(currentDateISO);

	let showDateFromPicker = $state(false);
	let showDateToPicker = $state(false);
	let pickerView = $state<'day' | 'month' | 'year'>('day');
	let slideDirection = $state<'left' | 'right'>('left');

	let calendarDate = $state(new Date());
	let activePickerIsFrom = $state(true);

	let reportGenerated = $state(false);
	let arrangeBy = $state<'date' | 'logType'>('date');
	let isLoading = $state(false);
	let error = $state<string | null>(null);
	let logEntries = $state<LogEntry[]>([]);
	let filteredEntries = $state<LogEntry[]>([]);
	let availableLogTypes = $state<Set<string>>(new Set());

	// Function to parse entry data and extract readable values
	function parseEntryData(
		entryData: unknown,
		templateLayout: any[],
		excludeFieldTypes: string[] = []
	): string {
		if (!entryData) return 'No data available';

		try {
			// Parse the entry data if it's a string
			let data: any;
			if (typeof entryData === 'string') {
				try {
					data = JSON.parse(entryData);
				} catch {
					// If it fails to parse as JSON, treat as plain text
					return entryData;
				}
			} else if (typeof entryData === 'object') {
				data = entryData;
			} else {
				return String(entryData);
			}

			if (!templateLayout || templateLayout.length === 0) {
				// If no template layout, try to display the raw data in a readable format
				if (typeof data === 'object' && data !== null) {
					const entries = Object.entries(data)
						.filter(([key, value]) => value !== null && value !== undefined && value !== '')
						.map(([key, value]) => `${key}: ${value}`);
					return entries.length > 0 ? entries.join(', ') : 'No data entered';
				}
				return String(data);
			}

			const results: string[] = [];

			// Try different possible field identifier patterns
			templateLayout.forEach((field: any, index: number) => {
				// Skip fields of excluded types - handle null/undefined as text fields
				const fieldType = field.field_type;
				const isExcluded = excludeFieldTypes.includes(fieldType) || 
				                  (!fieldType && excludeFieldTypes.includes(''));
				if (isExcluded) {
					return; // Skip this field
				}

				// Try various ways the field ID might be stored
				const possibleIds = [
					field.field_id,
					field.id,
					field.name,
					field.props?.name,
					field.props?.id,
					`field_${index}`,
					index.toString()
				].filter(Boolean);

				let fieldValue: any;
				let fieldId: string | undefined;

				// Try to find the field value using any of the possible IDs
				for (const id of possibleIds) {
					if (data[id] !== undefined && data[id] !== null && data[id] !== '') {
						fieldValue = data[id];
						fieldId = id;
						break;
					}
				}

				// If we found a value, format it appropriately
				if (fieldValue !== undefined && fieldId) {
					const fieldType = field.field_type;
					let displayValue = fieldValue;

					// Format different field types
					if (fieldType === 'temperature' && typeof fieldValue === 'number') {
						displayValue = `${fieldValue}°C`;
					} else if (fieldType === 'checkbox' || fieldType === 'boolean') {
						displayValue = fieldValue ? 'Yes' : 'No';
					} else if (fieldType === 'dropdown' || fieldType === 'select') {
						// Try to find the option label
						const options = field.props?.options || field.options;
						if (Array.isArray(options)) {
							const option = options.find(
								(opt: any) => opt.value === fieldValue || opt.id === fieldValue
							);
							displayValue = option?.label || option?.text || fieldValue;
						}
					}

					// Get field label
					const fieldLabel = field.props?.label || field.label || field.name || `Field ${fieldId}`;
					results.push(`${fieldLabel}: ${displayValue}`);
				}
			});

			// If no results from template parsing, try to show raw data
			if (results.length === 0 && typeof data === 'object' && data !== null) {
				const entries = Object.entries(data)
					.filter(([key, value]) => value !== null && value !== undefined && value !== '')
					.map(([key, value]) => `${key}: ${value}`);
				return entries.length > 0 ? entries.join(', ') : 'No data entered';
			}

			return results.length > 0 ? results.join(', ') : 'No data entered';
		} catch (e) {
			console.error('Error parsing entry data:', e);
			return 'Error parsing data';
		}
	}

	// Function to categorize log type based on template layout
	function categorizeLogType(templateLayout: any[]): string {
		if (!templateLayout || templateLayout.length === 0) return 'Text Logs';

		const fieldTypes = templateLayout.map((field) => field.field_type).filter(Boolean);
		console.log('Categorizing log with field types:', fieldTypes);

		if (fieldTypes.includes('temperature')) return 'Temperature Logs';
		if (fieldTypes.includes('checkbox')) return 'Checkbox Logs';
		if (fieldTypes.includes('dropdown')) return 'Dropdown Logs';

		return 'Text Logs';
	}

	// Function to get field type from a single field
	function getFieldType(field: any): string {
		const fieldType = field.field_type;
		if (fieldType === 'temperature') return 'Temperature Logs';
		if (fieldType === 'checkbox') return 'Checkbox Logs';
		if (fieldType === 'dropdown') return 'Dropdown Logs';
		return 'Text Logs';
	}

	// Function to extract individual components from entries
	function extractComponents(entries: LogEntry[], excludedFieldTypes: string[]) {
		const components: Array<{
			entry: LogEntry;
			field: any;
			fieldType: string;
			fieldData: string;
			fieldIndex: number;
		}> = [];

		entries.forEach((entry) => {
			if (!entry.template_layout) return;

			entry.template_layout.forEach((field: any, index: number) => {
const fieldType = field.field_type;

				// Skip excluded field types - handle null/undefined as text fields
				if (excludedFieldTypes.includes(fieldType || '') || (!fieldType && excludedFieldTypes.includes(''))) return;

				// Parse the field data
				const fieldData = parseFieldData(entry.entry_data, field, index);

				// Only add if field has data
				if (fieldData && fieldData !== 'No data entered' && fieldData !== 'No data available') {
					components.push({
						entry,
						field,
						fieldType: getFieldType(field),
						fieldData,
						fieldIndex: index
					});
				}
			});
		});

		return components;
	}

	// Function to parse individual field data
	function parseFieldData(entryData: unknown, field: any, fieldIndex: number): string {
		if (!entryData) return '';

		try {
			let data: any;
			if (typeof entryData === 'string') {
				try {
					data = JSON.parse(entryData);
				} catch {
					return '';
				}
			} else if (typeof entryData === 'object') {
				data = entryData;
			} else {
				return '';
			}

			// Try various field identifiers
			const possibleIds = [
				field.field_id,
				field.id,
				field.name,
				field.props?.name,
				field.props?.id,
				`field_${fieldIndex}`,
				fieldIndex.toString()
			].filter(Boolean);

			let fieldValue: any;
			for (const id of possibleIds) {
				if (data[id] !== undefined && data[id] !== null && data[id] !== '') {
					fieldValue = data[id];
					break;
				}
			}

			if (fieldValue === undefined) return '';

			// Format the field value
			const fieldType = field.field_type;
			let displayValue = fieldValue;

			if (fieldType === 'temperature' && typeof fieldValue === 'number') {
				displayValue = `${fieldValue}°C`;
			} else if (fieldType === 'checkbox' || fieldType === 'boolean') {
				displayValue = fieldValue ? 'Yes' : 'No';
			} else if (fieldType === 'dropdown' || fieldType === 'select') {
				const options = field.props?.options || field.options;
				if (Array.isArray(options)) {
					const option = options.find(
						(opt: any) => opt.value === fieldValue || opt.id === fieldValue
					);
					displayValue = option?.label || option?.text || fieldValue;
				}
			}

			const fieldLabel = field.props?.label || field.label || field.name || `Field ${fieldIndex}`;
			return `${fieldLabel}: ${displayValue}`;
		} catch (e) {
			return '';
		}
	}

	// Function to get excluded field types based on checkbox selections
	function getExcludedFieldTypes(): string[] {
		const excludedTypes: string[] = [];

		// Check which log types are NOT selected
		logTypes.forEach((type) => {
			if (type.id !== 'all' && !type.checked) {
				// Map log type labels to field types
				if (type.label === 'Temperature Logs') {
					excludedTypes.push('temperature');
				} else if (type.label === 'Checkbox Logs') {
					excludedTypes.push('checkbox');
				} else if (type.label === 'Dropdown Logs') {
					excludedTypes.push('dropdown');
				} else if (type.label === 'Text Logs') {
					// Text logs include fields with no field_type or undefined/null field_type
					excludedTypes.push('text', 'input', '');
				}
			}
		});

		console.log('Excluded field types:', excludedTypes);
		return excludedTypes;
	}

	// Function to check if an entry has any remaining fields after filtering
	function hasRemainingFields(templateLayout: any[], excludeFieldTypes: string[], entryData: unknown = null): boolean {
		if (!templateLayout || templateLayout.length === 0) return true;
		
		// Debug logging
		console.log('Checking fields for entry:', {
			templateLayout: templateLayout?.map(f => ({ type: f.field_type, id: f.field_id || f.id })),
			excludedTypes: excludeFieldTypes
		});
		
		// Check if at least one field is not excluded AND has actual data
		const result = templateLayout.some((field, index) => {
			const fieldType = field.field_type;
			
			console.log(`Checking field ${index}: type='${fieldType}', excluded=${excludeFieldTypes.includes(fieldType || '') || (!fieldType && excludeFieldTypes.includes(''))}`);
			
			// Skip excluded field types (handle null/undefined/empty as text fields)
			const isExcluded = excludeFieldTypes.includes(fieldType || '') || 
			                  (!fieldType && excludeFieldTypes.includes(''));
			
			if (isExcluded) {
				console.log(`Field excluded (type: ${fieldType})`);
				return false;
			}
			
			// If no entry data provided, just check field type existence
			if (!entryData) {
				console.log(`Field accepted (type: ${fieldType}) - no data check`);
				return true;
			}
			
			// Check if this field actually has data
			const fieldData = parseFieldData(entryData, field, index);
			if (!fieldData) {
				console.log(`Field rejected (type: ${fieldType}) - no data`);
				return false;
			}
			
			// Convert to string and check if it's meaningful content
			const dataStr = String(fieldData);
			const hasValidData = dataStr !== 'No data entered' && 
			                    dataStr !== 'No data available' && 
			                    dataStr.trim() !== '' && 
			                    dataStr !== 'undefined' && 
			                    dataStr !== 'null';
			
			console.log(`Field ${hasValidData ? 'accepted' : 'rejected'} (type: ${fieldType}) - data: "${dataStr}"`);
			return hasValidData;
		});
		
		console.log('hasRemainingFields result:', result);
		return result;
	}

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

	function handleAllCheckboxChange(checked: boolean) {
		logTypes.forEach((logType) => {
			if (logType.id !== 'all') {
				logType.checked = checked;
			}
		});
	}

	function handleIndividualCheckboxChange() {
		const allCheckbox = logTypes.find((logType) => logType.id === 'all');
		const otherCheckboxes = logTypes.filter((logType) => logType.id !== 'all');
		const allOthersChecked = otherCheckboxes.every((logType) => logType.checked);

		if (allCheckbox) {
			allCheckbox.checked = allOthersChecked;
		}
	}

	async function generateReport() {
		isLoading = true;
		error = null;
		reportGenerated = false;

		try {
			// Fetch all log entries for the user's company
			const response = await api.GET('/logs/entries');

			if (!response.data) {
				throw new Error('Failed to fetch log entries');
			}

			logEntries = response.data.entries;

			// Collect available log types from actual data
			const logTypesSet = new Set<string>();
			logEntries.forEach((entry) => {
				const category = categorizeLogType(entry.template_layout);
				logTypesSet.add(category);
			});
			availableLogTypes = logTypesSet;

			// Filter entries based on date range and selected log types
			const selectedLogTypes = logTypes
				.filter((type) => type.checked && type.id !== 'all')
				.map((type) => type.label);

			// If "All" is selected or no specific types selected, show all types
			const showAllTypes =
				logTypes.find((type) => type.id === 'all')?.checked || selectedLogTypes.length === 0;

			// Get excluded field types for filtering
			const excludedFieldTypes = getExcludedFieldTypes();

			filteredEntries = logEntries.filter((entry) => {
				// Filter by date range
				const entryDate = new Date(entry.created_at);
				const fromDate = new Date(dateFromISO);
				const toDate = new Date(dateToISO);
				// Set time to end of day for 'to' date
				toDate.setHours(23, 59, 59, 999);

				const isInDateRange = entryDate >= fromDate && entryDate <= toDate;

				// Check if entry has any remaining fields after filtering
				const hasFields = hasRemainingFields(entry.template_layout, excludedFieldTypes, entry.entry_data);
				
				// Debug logging for troubleshooting
				if (isInDateRange && !hasFields) {
					console.log('Entry excluded due to no remaining fields:', {
						entryId: entry.id.slice(0, 8),
						templateName: entry.template_name,
						excludedTypes: excludedFieldTypes,
						templateLayout: entry.template_layout?.map(f => f.field_type)
					});
				}

				return isInDateRange && hasFields;
			});

			// Sort entries based on arrange preference
			if (arrangeBy === 'date') {
				filteredEntries.sort(
					(a, b) => new Date(b.created_at).getTime() - new Date(a.created_at).getTime()
				);
			} else {
				// For log type sorting, we'll handle grouping in the display
				filteredEntries.sort((a, b) => {
					if (a.template_name < b.template_name) return -1;
					if (a.template_name > b.template_name) return 1;
					// If same template, sort by date
					return new Date(b.created_at).getTime() - new Date(a.created_at).getTime();
				});
			}

			reportGenerated = true;
		} catch (err) {
			error = err instanceof Error ? err.message : 'An error occurred while generating the report';
		} finally {
			isLoading = false;
		}
	}

	// Export functions
	function exportToPDF() {
		if (!reportGenerated || filteredEntries.length === 0) return;

		const reportContent = generateTextReportContent();
		const printWindow = window.open('', '_blank');

		if (printWindow) {
			printWindow.document.write(`
				<html>
				<head>
					<title>Log Report</title>
					<style>
						body { font-family: Arial, sans-serif; margin: 20px; }
						.header { border-bottom: 2px solid #333; margin-bottom: 20px; padding-bottom: 10px; }
						.entry { border: 1px solid #ddd; margin: 10px 0; padding: 15px; border-radius: 5px; page-break-inside: avoid; }
						.status { padding: 3px 8px; border-radius: 3px; color: white; font-size: 12px; }
						.submitted { background-color: #10B981; }
						.draft { background-color: #F59E0B; }
						.group-header { font-size: 18px; font-weight: bold; margin: 20px 0 10px 0; border-bottom: 1px solid #ccc; padding-bottom: 5px; }
						.entry-data { background-color: #f5f5f5; padding: 10px; margin: 5px 0; border-radius: 3px; }
						@media print {
							body { margin: 0; }
							.entry { page-break-inside: avoid; }
						}
					</style>
				</head>
				<body>
					${reportContent}
				</body>
				</html>
			`);
			printWindow.document.close();
			printWindow.print();
		}
	}

	function exportToWord(format: 'docx' | 'rtf') {
		if (!reportGenerated || filteredEntries.length === 0) return;

		let content: string;
		let mimeType: string;
		let filename: string;

		if (format === 'rtf') {
			content = generateRTFContent();
			mimeType = 'application/rtf';
			filename = 'log-report.rtf';
		} else {
			// For DOCX, create a proper HTML document that Word can import
			content = generateWordHTMLContent();
			mimeType = 'application/vnd.openxmlformats-officedocument.wordprocessingml.document';
			filename = 'log-report.doc'; // Use .doc extension for better compatibility
		}

		const blob = new Blob([content], { type: mimeType });
		const url = URL.createObjectURL(blob);
		const a = document.createElement('a');
		a.href = url;
		a.download = filename;
		a.click();
		URL.revokeObjectURL(url);
	}

	function generateWordHTMLContent(): string {
		return `
<!DOCTYPE html>
<html xmlns:o="urn:schemas-microsoft-com:office:office" xmlns:w="urn:schemas-microsoft-com:office:word" xmlns="http://www.w3.org/TR/REC-html40">
<head>
	<meta charset="utf-8">
	<meta name="ProgId" content="Word.Document">
	<meta name="Generator" content="Microsoft Word">
	<meta name="Originator" content="Microsoft Word">
	<title>Log Report</title>
	<style>
		body { font-family: 'Times New Roman', serif; font-size: 12pt; margin: 1in; }
		h1 { font-size: 16pt; font-weight: bold; text-align: center; margin-bottom: 12pt; }
		h2 { font-size: 14pt; font-weight: bold; margin-top: 12pt; margin-bottom: 6pt; }
		.header { border-bottom: 1pt solid black; padding-bottom: 6pt; margin-bottom: 12pt; }
		.entry { border: 1pt solid #ccc; margin: 6pt 0; padding: 8pt; }
		.entry-header { font-weight: bold; margin-bottom: 4pt; }
		.entry-data { background-color: #f0f0f0; padding: 4pt; margin: 4pt 0; }
		.status { padding: 2pt 4pt; border-radius: 2pt; color: white; font-size: 10pt; }
		.submitted { background-color: #10B981; }
		.draft { background-color: #F59E0B; }
		table { width: 100%; border-collapse: collapse; margin: 6pt 0; table-layout: fixed; border: 1pt solid #000; }
		td { padding: 6pt; border: 1pt solid #000; vertical-align: top; word-wrap: break-word; font-size: 11pt; }
		.label { font-weight: bold; width: 30%; background-color: #f0f0f0; }
		.value { width: 70%; }
	</style>
</head>
<body>
	<div class="header">
		<h1>Log Report</h1>
		<p><strong>Date Range:</strong> ${dateFrom} - ${dateTo}</p>
		<p><strong>Arranged by:</strong> ${arrangeBy === 'date' ? 'Date' : 'Log Type'}</p>
		<p><strong>Total Entries:</strong> ${filteredEntries.length}</p>
		<p><strong>Generated on:</strong> ${new Date().toLocaleString()}</p>
	</div>
	
	${generateWordEntries()}
</body>
</html>`;
	}

	function generateWordEntries(): string {
		let content = '';

		if (arrangeBy === 'logType') {
			const groupedEntries = filteredEntries.reduce((acc: Record<string, LogEntry[]>, entry) => {
				const category = categorizeLogType(entry.template_layout);
				if (!acc[category]) acc[category] = [];
				acc[category].push(entry);
				return acc;
			}, {});

			Object.entries(groupedEntries).forEach(([category, entries]) => {
				content += `<h2>${category} (${entries.length} entries)</h2>`;
				entries.forEach((entry) => {
					content += generateWordEntryHTML(entry);
				});
			});
		} else {
			filteredEntries.forEach((entry) => {
				content += generateWordEntryHTML(entry);
			});
		}

		return content;
	}

	function generateWordEntryHTML(entry: LogEntry): string {
		const excludedFieldTypes = getExcludedFieldTypes();
		const entryData = parseEntryData(entry.entry_data, entry.template_layout, excludedFieldTypes);

		return `
		<div class="entry">
			<div class="entry-header">
				${entry.template_name}
				<span class="status ${entry.status === 'submitted' ? 'submitted' : 'draft'}">${entry.status}</span>
				<br><small>ID: ${entry.id.slice(0, 8)}...</small>
			</div>
			
			<table>
				<tr>
					<td class="label">Entry Data:</td>
					<td class="value">${entryData}</td>
				</tr>
				<tr>
					<td class="label">Created:</td>
					<td class="value">${new Date(entry.created_at).toLocaleString()}</td>
				</tr>
				${entry.submitted_at ? `<tr><td class="label">Submitted:</td><td class="value">${new Date(entry.submitted_at).toLocaleString()}</td></tr>` : ''}
				<tr>
					<td class="label">Period:</td>
					<td class="value">${entry.period}</td>
				</tr>
			</table>
		</div>
		`;
	}

	function generateRTFContent(): string {
		// Generate properly formatted RTF
		let rtfContent = `{\\rtf1\\ansi\\deff0{\\fonttbl{\\f0\\froman Times New Roman;}{\\f1\\fswiss Arial;}}
{\\colortbl;\\red0\\green0\\blue0;\\red16\\green185\\blue129;\\red245\\green158\\blue11;}
\\f0\\fs24`;

		// Header
		rtfContent += `\\qc\\b\\fs28 Log Report\\b0\\fs24\\par\\par`;
		rtfContent += `\\ql\\b Date Range: \\b0 ${dateFrom} - ${dateTo}\\par`;
		rtfContent += `\\b Arranged by: \\b0 ${arrangeBy === 'date' ? 'Date' : 'Log Type'}\\par`;
		rtfContent += `\\b Total Entries: \\b0 ${filteredEntries.length}\\par`;
		rtfContent += `\\b Generated on: \\b0 ${new Date().toLocaleString()}\\par`;
		rtfContent += `\\par\\pard\\brdrb\\brdrs\\brdrw10\\par\\par`;

		if (arrangeBy === 'logType') {
			const groupedEntries = filteredEntries.reduce((acc: Record<string, LogEntry[]>, entry) => {
				const category = categorizeLogType(entry.template_layout);
				if (!acc[category]) acc[category] = [];
				acc[category].push(entry);
				return acc;
			}, {});

			Object.entries(groupedEntries).forEach(([category, entries]) => {
				rtfContent += `\\b\\fs26 ${category} (${entries.length} entries)\\b0\\fs24\\par\\par`;
				entries.forEach((entry) => {
					rtfContent += generateRTFEntry(entry);
				});
			});
		} else {
			filteredEntries.forEach((entry) => {
				rtfContent += generateRTFEntry(entry);
			});
		}

		rtfContent += '}';
		return rtfContent;
	}

	function generateRTFEntry(entry: LogEntry): string {
		const excludedFieldTypes = getExcludedFieldTypes();
		const entryData = parseEntryData(entry.entry_data, entry.template_layout, excludedFieldTypes);
		const statusColor = entry.status === 'submitted' ? '\\cf2' : '\\cf3';

		let rtf = `\\pard\\box\\brdrs\\brdrw10\\brdrcf1\\par`;
		rtf += `\\b ${entry.template_name}\\b0\\tab ${statusColor}${entry.status}\\cf1\\par`;
		rtf += `\\fi0\\b Created:\\b0 ${new Date(entry.created_at).toLocaleString()}\\par`;
		if (entry.submitted_at) {
			rtf += `\\b Submitted:\\b0 ${new Date(entry.submitted_at).toLocaleString()}\\par`;
		}
		rtf += `\\b Period:\\b0 ${entry.period}\\par`;
		rtf += `\\par\\pard\\par`;

		return rtf;
	}

	function generateTextReportContent(): string {
		let content = `
			<div class="header">
				<h1>Log Report</h1>
				<p><strong>Date Range:</strong> ${dateFrom} - ${dateTo}</p>
				<p><strong>Arranged by:</strong> ${arrangeBy === 'date' ? 'Date' : 'Log Type'}</p>
				<p><strong>Total Entries:</strong> ${filteredEntries.length}</p>
				<p><strong>Generated on:</strong> ${new Date().toLocaleString()}</p>
			</div>
		`;

		if (arrangeBy === 'logType') {
			const groupedEntries = filteredEntries.reduce((acc: Record<string, LogEntry[]>, entry) => {
				const category = categorizeLogType(entry.template_layout);
				if (!acc[category]) acc[category] = [];
				acc[category].push(entry);
				return acc;
			}, {});

			Object.entries(groupedEntries).forEach(([category, entries]) => {
				content += `<div class="group-header">${category} (${entries.length} entries)</div>`;
				entries.forEach((entry) => {
					content += generateEntryHTML(entry);
				});
			});
		} else {
			filteredEntries.forEach((entry) => {
				content += generateEntryHTML(entry);
			});
		}

		return content;
	}

	function generateReportContent(): string {
		let content = `
			<div class="header">
				<h1>Log Report</h1>
				<p><strong>Date Range:</strong> ${dateFrom} - ${dateTo}</p>
				<p><strong>Arranged by:</strong> ${arrangeBy === 'date' ? 'Date' : 'Log Type'}</p>
				<p><strong>Total Entries:</strong> ${filteredEntries.length}</p>
				<p><strong>Generated on:</strong> ${new Date().toLocaleString()}</p>
			</div>
		`;

		if (arrangeBy === 'logType') {
			const groupedEntries = filteredEntries.reduce((acc: Record<string, LogEntry[]>, entry) => {
				const category = categorizeLogType(entry.template_layout);
				if (!acc[category]) acc[category] = [];
				acc[category].push(entry);
				return acc;
			}, {});

			Object.entries(groupedEntries).forEach(([category, entries]) => {
				content += `<div class="group-header">${category} (${entries.length} entries)</div>`;
				entries.forEach((entry) => {
					content += generateEntryHTML(entry);
				});
			});
		} else {
			filteredEntries.forEach((entry) => {
				content += generateEntryHTML(entry);
			});
		}

		return content;
	}

	function generateEntryHTML(entry: LogEntry): string {
		const excludedFieldTypes = getExcludedFieldTypes();
		const entryData = parseEntryData(entry.entry_data, entry.template_layout, excludedFieldTypes);
		return `
			<div class="entry">
				<div style="display: flex; justify-content: space-between; align-items: start; margin-bottom: 10px;">
					<div>
						<strong>${entry.template_name}</strong>
						<small style="color: #666; margin-left: 10px;">ID: ${entry.id.slice(0, 8)}...</small>
					</div>
					<span class="status ${entry.status === 'submitted' ? 'submitted' : 'draft'}">${entry.status}</span>
				</div>
				<div class="entry-data">
					<strong>Entry Data:</strong> ${entryData}
				</div>
				<p><strong>Created:</strong> ${new Date(entry.created_at).toLocaleString()}</p>
				${entry.submitted_at ? `<p><strong>Submitted:</strong> ${new Date(entry.submitted_at).toLocaleString()}</p>` : ''}
				<p><strong>Period:</strong> ${entry.period}</p>
			</div>
		`;
	}
</script>

<svelte:head>
	<title>Generate Report</title>
</svelte:head>
<div class="min-h-full" style="background-color: var(--bg-secondary);">
	<!-- Main Content -->
	<div class="mx-auto max-w-7xl px-6 py-8">
		<h1 class="mb-8 text-center text-3xl font-bold md:text-4xl" style="color: var(--text-primary);">
			Generate Report
		</h1>

		<div class="flex flex-col gap-8 lg:flex-row lg:gap-8">
			<!-- Left Side - Form -->
			<div class="w-full lg:w-96">
				<!-- Date From -->
				<div class="mb-8">
					<label
						for="date-from"
						class="mb-3 block text-lg font-bold"
						style="color: var(--text-primary);">Date From:</label
					>
					<div class="relative">
						<div class="flex items-center gap-2">
							<input
								id="date-from"
								type="text"
								bind:value={dateFrom}
								oninput={(e) => updateDateFromText(e.currentTarget.value, true)}
								placeholder="DD/MM/YYYY"
								class="flex-1 border-2 px-4 py-2"
								style="border-color: var(--border-primary); background-color: var(--bg-primary); color: var(--text-primary);"
							/>
							<button
								type="button"
								onclick={() => toggleDatePicker(true)}
								aria-label="Open calendar for start date"
								class="border-2 p-2"
								style="border-color: var(--border-primary); background-color: var(--bg-primary); color: var(--text-primary);"
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

						<!-- Modern Date Picker -->
						{#if showDateFromPicker}
							<div
								class="absolute top-full left-0 z-50 mt-2 rounded-lg border-2 p-4 shadow-lg"
								style="border-color: var(--border-primary); background-color: var(--bg-primary); min-width: 280px; sm:min-width: 320px; overflow: hidden; right: auto;"
							>
								<!-- Day View -->
								{#if pickerView === 'day'}
									<div class={slideDirection === 'left' ? 'slide-left' : 'slide-right'}>
										<!-- Month/Year Header -->
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
													stroke-width="2"
												>
													<polyline points="12 4 6 10 12 16"></polyline>
												</svg>
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
													stroke-width="2"
												>
													<polyline points="8 4 14 10 8 16"></polyline>
												</svg>
											</button>
										</div>

										<!-- Day Labels -->
										<div class="mb-2 grid grid-cols-7 gap-1">
											{#each ['Su', 'Mo', 'Tu', 'We', 'Th', 'Fr', 'Sa'] as day}
												<div class="py-2 text-center text-sm font-medium" style="color: #A1A6B4;">
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

								<!-- Month View -->
								{#if pickerView === 'month'}
									<div class={slideDirection === 'left' ? 'slide-left' : 'slide-right'}>
										<!-- Year Header -->
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
													stroke-width="2"
												>
													<polyline points="12 4 6 10 12 16"></polyline>
												</svg>
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
													stroke-width="2"
												>
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

								<!-- Year View -->
								{#if pickerView === 'year'}
									<div class={slideDirection === 'left' ? 'slide-left' : 'slide-right'}>
										<!-- Year Range Header -->
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
													stroke-width="2"
												>
													<polyline points="12 4 6 10 12 16"></polyline>
												</svg>
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
													stroke-width="2"
												>
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
				<div class="mb-8">
					<label
						for="date-to"
						class="mb-3 block text-lg font-bold"
						style="color: var(--text-primary);">Date To:</label
					>
					<div class="relative">
						<div class="flex items-center gap-2">
							<input
								id="date-to"
								type="text"
								bind:value={dateTo}
								oninput={(e) => updateDateFromText(e.currentTarget.value, false)}
								placeholder="DD/MM/YYYY"
								class="flex-1 border-2 px-4 py-2"
								style="border-color: var(--border-primary); background-color: var(--bg-primary); color: var(--text-primary);"
							/>
							<button
								type="button"
								onclick={() => toggleDatePicker(false)}
								aria-label="Open calendar for end date"
								class="border-2 p-2"
								style="border-color: var(--border-primary); background-color: var(--bg-primary); color: var(--text-primary);"
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

						<!-- Modern Date Picker -->
						{#if showDateToPicker}
							<div
								class="absolute top-full left-0 z-50 mt-2 rounded-lg border-2 p-4 shadow-lg"
								style="border-color: var(--border-primary); background-color: var(--bg-primary); min-width: 280px; sm:min-width: 320px; overflow: hidden; right: auto;"
							>
								<!-- Day View -->
								{#if pickerView === 'day'}
									<div class={slideDirection === 'left' ? 'slide-left' : 'slide-right'}>
										<!-- Month/Year Header -->
										<div class="mb-4 flex items-center justify-between">
											<button
												type="button"
												onclick={previousMonth}
												aria-label="Previous month"
												class="rounded p-2"
												style="color: var(--text-primary); background-color: transparent;"
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
												class="rounded px-3 py-1 font-bold transition-colors"
												style="color: var(--text-primary); background-color: transparent;"
											>
												{monthNames[calendarDate.getMonth()]}
												{calendarDate.getFullYear()}
											</button>
											<button
												type="button"
												onclick={nextMonth}
												aria-label="Next month"
												class="rounded p-2"
												style="color: var(--text-primary); background-color: transparent;"
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
										<!-- Day Labels -->
										<div class="mb-2 grid grid-cols-7 gap-1">
											{#each ['Su', 'Mo', 'Tu', 'We', 'Th', 'Fr', 'Sa'] as day}
												<div
													class="py-2 text-center text-sm font-medium"
													style="color: var(--text-secondary);"
												>
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
														class="flex aspect-square items-center justify-center rounded transition-colors"
														class:font-bold={isSelectedDay(day)}
														style={isSelectedDay(day)
															? 'background-color: #3D7A82; color: white;'
															: 'color: var(--text-primary); background-color: transparent;'}
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
										<div class="mb-4 flex items-center justify-between">
											<button
												type="button"
												onclick={previousYear}
												aria-label="Previous year"
												class="rounded p-2 hover:bg-gray-100"
												style="color: #000100;"
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
												class="rounded px-3 py-1 font-bold transition-colors hover:bg-gray-100"
												style="color: #000100;"
											>
												{calendarDate.getFullYear()}
											</button>
											<button
												type="button"
												onclick={nextYear}
												aria-label="Next year"
												class="rounded p-2 hover:bg-gray-100"
												style="color: #000100;"
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

										<!-- Month Grid -->
										<div class="grid grid-cols-3 gap-2">
											{#each monthNamesShort as month, index}
												<button
													type="button"
													onclick={() => selectMonth(index)}
													class="rounded px-4 py-3 font-medium transition-colors hover:bg-gray-100"
													class:font-bold={calendarDate.getMonth() === index}
													style={calendarDate.getMonth() === index
														? 'background-color: #3D7A82; color: white;'
														: 'color: #000100;'}
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
										<div class="mb-4 flex items-center justify-between">
											<button
												type="button"
												onclick={previousYearRange}
												aria-label="Previous years"
												class="rounded p-2 hover:bg-gray-100"
												style="color: #000100;"
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
											<div class="font-bold" style="color: #000100;">
												{getYearRange()[0]} - {getYearRange()[11]}
											</div>
											<button
												type="button"
												onclick={nextYearRange}
												aria-label="Next years"
												class="rounded p-2 hover:bg-gray-100"
												style="color: #000100;"
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

										<!-- Year Grid -->
										<div class="grid grid-cols-3 gap-2">
											{#each getYearRange() as year}
												<button
													type="button"
													onclick={() => selectYear(year)}
													class="rounded px-4 py-3 font-medium transition-colors hover:bg-gray-100"
													class:font-bold={calendarDate.getFullYear() === year}
													style={calendarDate.getFullYear() === year
														? 'background-color: #3D7A82; color: white;'
														: 'color: #000100;'}
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
						<legend class="mb-3 block text-lg font-bold" style="color: var(--text-primary);"
							>Log Types:</legend
						>
						<div class="space-y-2">
							{#each logTypes as logType}
								<label class="flex cursor-pointer items-center gap-3">
									<input
										type="checkbox"
										bind:checked={logType.checked}
										onchange={(e) =>
											logType.id === 'all'
												? handleAllCheckboxChange(e.currentTarget.checked)
												: handleIndividualCheckboxChange()}
										class="h-5 w-5 cursor-pointer border-2"
										style="border-color: var(--border-primary);"
									/>
									<span style="color: var(--text-primary);">{logType.label}</span>
								</label>
							{/each}
						</div>
					</fieldset>
				</div>

				<!-- Arrange By Options -->
				<div class="mb-8">
					<legend class="mb-3 block text-lg font-bold" style="color: var(--text-primary);"
						>Arrange By:</legend
					>
					<div class="flex gap-3">
						<button
							type="button"
							onclick={() => (arrangeBy = 'date')}
							class="flex-1 border-2 px-4 py-2 font-bold transition-all"
							style={arrangeBy === 'date'
								? 'border-color: #3D7A82; background-color: #3D7A82; color: white; box-shadow: 0 0 8px rgba(61, 122, 130, 0.3);'
								: 'border-color: var(--border-primary); background-color: transparent; color: var(--text-secondary);'}
						>
							Arrange By Date
						</button>
						<button
							type="button"
							onclick={() => (arrangeBy = 'logType')}
							class="flex-1 border-2 px-4 py-2 font-bold transition-all"
							style={arrangeBy === 'logType'
								? 'border-color: #3D7A82; background-color: #3D7A82; color: white; box-shadow: 0 0 8px rgba(61, 122, 130, 0.3);'
								: 'border-color: var(--border-primary); background-color: transparent; color: var(--text-secondary);'}
						>
							Arrange By Log Type
						</button>
					</div>
				</div>

				<!-- Generate Button -->
				<div class="flex justify-center">
					<button
						onclick={generateReport}
						class="flex items-center gap-2 border-2 px-8 py-2 font-medium hover:opacity-80"
						style="border-color: var(--border-primary); background-color: var(--bg-primary); color: var(--text-primary);"
					>
						Generate
						<svg
							width="16"
							height="16"
							viewBox="0 0 16 16"
							fill="none"
							stroke="currentColor"
							stroke-width="2"
						>
							<polyline points="6 3 11 8 6 13"></polyline>
						</svg>
					</button>
				</div>
			</div>

			<!-- Right Side - Report Preview -->
			<div class="w-full lg:flex-1">
				<!-- Download Buttons -->
				<div class="mb-4 flex flex-col gap-2 sm:flex-row sm:justify-end sm:gap-4">
					<button
						onclick={exportToPDF}
						disabled={!reportGenerated || filteredEntries.length === 0}
						class="border-2 px-4 py-2 text-sm font-medium transition-all duration-200 transform hover:scale-105 hover:shadow-lg hover:bg-blue-50 hover:border-blue-400 hover:text-blue-700 disabled:cursor-not-allowed disabled:opacity-50 disabled:hover:transform-none disabled:hover:shadow-none sm:text-base"
						style="border-color: var(--border-primary); color: var(--text-primary); background-color: var(--bg-primary);"
					>
						📄 Download PDF
					</button>
					<button
						onclick={() => exportToWord('docx')}
						disabled={!reportGenerated || filteredEntries.length === 0}
						class="border-2 px-4 py-2 text-sm font-medium transition-all duration-200 transform hover:scale-105 hover:shadow-lg hover:bg-green-50 hover:border-green-400 hover:text-green-700 disabled:cursor-not-allowed disabled:opacity-50 disabled:hover:transform-none disabled:hover:shadow-none sm:text-base"
						style="border-color: var(--border-primary); color: var(--text-primary); background-color: var(--bg-primary);"
					>
						📊 Download DOCX
					</button>
					<button
						onclick={() => exportToWord('rtf')}
						disabled={!reportGenerated || filteredEntries.length === 0}
						class="border-2 px-4 py-2 text-sm font-medium transition-all duration-200 transform hover:scale-105 hover:shadow-lg hover:bg-purple-50 hover:border-purple-400 hover:text-purple-700 disabled:cursor-not-allowed disabled:opacity-50 disabled:hover:transform-none disabled:hover:shadow-none sm:text-base"
						style="border-color: var(--border-primary); color: var(--text-primary); background-color: var(--bg-primary);"
					>
						📝 Download RTF
					</button>
				</div>

				<!-- Report Preview Area -->
				<div
					class="min-h-100 border-2 p-4 sm:min-h-150 sm:p-8"
					style="border-color: var(--border-primary); background-color: var(--bg-primary);"
				>
					{#if isLoading}
						<div class="flex items-center justify-center py-8">
							<div
								class="h-8 w-8 animate-spin rounded-full border-b-2"
								style="border-color: var(--text-primary);"
							></div>
							<span class="ml-3" style="color: var(--text-primary);">Generating report...</span>
						</div>
					{:else if error}
						<div class="flex items-start gap-3 text-red-500">
							<svg
								width="24"
								height="24"
								viewBox="0 0 24 24"
								fill="none"
								stroke="currentColor"
								stroke-width="2"
							>
								<circle cx="12" cy="12" r="10"></circle>
								<line x1="15" y1="9" x2="9" y2="15"></line>
								<line x1="9" y1="9" x2="15" y2="15"></line>
							</svg>
							<div>
								<p class="font-bold">Error generating report</p>
								<p>{error}</p>
							</div>
						</div>
					{:else if reportGenerated}
						<div>
							<div class="mb-6">
								<h2 class="mb-2 text-xl font-bold" style="color: var(--text-primary);">
									Log Report
								</h2>
								<p class="text-sm" style="color: var(--text-secondary);">
									Date Range: {dateFrom} - {dateTo} | Arranged by: {arrangeBy === 'date'
										? 'Date'
										: 'Log Type'} | Total Entries: {filteredEntries.length}
								</p>
							</div>

							{#if filteredEntries.length === 0}
								<div class="py-8 text-center">
									<svg
										width="48"
										height="48"
										viewBox="0 0 24 24"
										fill="none"
										stroke="currentColor"
										stroke-width="1"
										class="mx-auto mb-4"
										style="color: var(--text-secondary);"
									>
										<circle cx="11" cy="11" r="8"></circle>
										<path d="21 21l-4.35-4.35"></path>
									</svg>
									<p class="text-lg font-medium" style="color: var(--text-secondary);">
										No log entries found
									</p>
									<p class="text-sm" style="color: var(--text-secondary);">
										Try adjusting your date range or log type filters.
									</p>
								</div>
							{:else if arrangeBy === 'logType'}
								{@const excludedFieldTypes = getExcludedFieldTypes()}
								{@const components = extractComponents(filteredEntries, excludedFieldTypes)}
								{@const groupedComponents = components.reduce(
									(acc: Record<string, typeof components>, component) => {
										if (!acc[component.fieldType]) acc[component.fieldType] = [];
										acc[component.fieldType].push(component);
										return acc;
									},
									{}
								)}
								{#each Object.entries(groupedComponents) as [fieldType, componentGroup]}
									<div class="mb-6">
										<h3
											class="mb-3 border-b pb-2 text-lg font-bold"
											style="color: var(--text-primary); border-color: var(--border-primary);"
										>
											{fieldType} ({componentGroup.length} components)
										</h3>
										{#each componentGroup as component}
											<div
												class="mb-4 rounded border p-4"
												style="border-color: var(--border-primary); background-color: var(--bg-secondary);"
											>
												<div class="mb-2 flex items-start justify-between">
													<div>
														<span class="font-medium" style="color: var(--text-primary);"
															>{component.entry.template_name}</span
														>
														<span class="ml-2 text-sm" style="color: var(--text-secondary);"
															>ID: {component.entry.id.slice(0, 8)}...</span
														>
													</div>
													<span
														class="rounded px-2 py-1 text-xs"
														style={component.entry.status === 'submitted'
															? 'background-color: #10B981; color: white;'
															: 'background-color: #F59E0B; color: white;'}
													>
														{component.entry.status}
													</span>
												</div>
												<div class="mb-2 rounded p-2" style="background-color: var(--bg-primary);">
													<p class="mb-1 text-sm font-medium" style="color: var(--text-primary);">
														Component Data:
													</p>
													<p class="text-sm" style="color: var(--text-secondary);">
														{component.fieldData}
													</p>
												</div>
												<p class="mb-2 text-sm" style="color: var(--text-secondary);">
													Created: {new Date(component.entry.created_at).toLocaleString()}
												</p>
												{#if component.entry.submitted_at}
													<p class="mb-2 text-sm" style="color: var(--text-secondary);">
														Submitted: {new Date(component.entry.submitted_at).toLocaleString()}
													</p>
												{/if}
												<p class="text-sm" style="color: var(--text-secondary);">
													Period: {component.entry.period}
												</p>
											</div>
										{/each}
									</div>
								{/each}
							{:else}
								{#each filteredEntries as entry}
									{@const excludedFieldTypes = getExcludedFieldTypes()}
									{@const shouldShowEntry = hasRemainingFields(entry.template_layout, excludedFieldTypes, entry.entry_data)}
									{#if shouldShowEntry}
										<div
											class="mb-4 rounded border p-4"
											style="border-color: var(--border-primary); background-color: var(--bg-secondary);"
										>
											<div class="mb-2 flex items-start justify-between">
												<div>
													<span class="font-medium" style="color: var(--text-primary);"
														>{entry.template_name}</span
													>
													<span class="ml-2 text-sm" style="color: var(--text-secondary);"
														>ID: {entry.id.slice(0, 8)}...</span
													>
												</div>
												<span
													class="rounded px-2 py-1 text-xs"
													style={entry.status === 'submitted'
														? 'background-color: #10B981; color: white;'
														: 'background-color: #F59E0B; color: white;'}
												>
													{entry.status}
												</span>
											</div>
											<div class="mb-2 rounded p-2" style="background-color: var(--bg-primary);">
												<p class="mb-1 text-sm font-medium" style="color: var(--text-primary);">
													Entry Data:
												</p>
												<p class="text-sm" style="color: var(--text-secondary);">
													{parseEntryData(
														entry.entry_data,
														entry.template_layout,
														excludedFieldTypes
													)}
												</p>
											</div>
											<p class="mb-2 text-sm" style="color: var(--text-secondary);">
												Created: {new Date(entry.created_at).toLocaleString()}
											</p>
											{#if entry.submitted_at}
												<p class="mb-2 text-sm" style="color: var(--text-secondary);">
													Submitted: {new Date(entry.submitted_at).toLocaleString()}
												</p>
											{/if}
											<p class="text-sm" style="color: var(--text-secondary);">
												Period: {entry.period}
											</p>
										</div>
									{/if}
								{/each}
							{/if}
						</div>
					{:else}
						<div class="flex items-start gap-3">
							<svg
								width="32"
								height="32"
								viewBox="0 0 32 32"
								fill="none"
								stroke="currentColor"
								stroke-width="2"
								style="color: var(--text-secondary);"
							>
								<polyline points="9 17 14 22 23 10"></polyline>
								<rect x="4" y="4" width="24" height="24" rx="2" ry="2"></rect>
							</svg>
							<p style="color: var(--text-secondary);">Generate a report to preview</p>
						</div>
					{/if}
				</div>
			</div>
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
