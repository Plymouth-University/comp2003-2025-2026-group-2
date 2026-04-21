<script lang="ts">
	import { api } from '$lib/api';
	import type { components } from '$lib/api-types';
	import DatePicker from '$lib/components/DatePicker.svelte';
	import { onMount } from 'svelte';
	import { SvelteDate, SvelteMap, SvelteSet } from 'svelte/reactivity';
	import { PDF_STYLES } from '$lib/utils/pdf-templates';

	type LogEntry = components['schemas']['LogEntryResponse'];
	type TemplateField = components['schemas']['TemplateField'];

	type LogComponent = {
		entry: LogEntry;
		field: TemplateField;
		fieldType: string;
		fieldData: string;
		fieldIndex: number;
		componentId: string;
	};

	type SavedReportParams = {
		date_from_iso: string;
		date_to_iso: string;
		selected_branch_ids: string[];
		selected_log_type_ids: string[];
		arrange_by: 'date' | 'logType';
		include_temperature_graphs: boolean;
		params_version: number;
	};

	type ReportRun = {
		id: string;
		name?: string | null;
		params: SavedReportParams;
		created_at: string;
		last_used_at: string;
		use_count: number;
	};

	// Get user data from parent layout
	let { data } = $props();
	let user = $derived(data?.user);
	let branches = $derived(data?.branches || []);

	// Check if user is readonly HQ (staff with no branch) or branch manager
	let isReadonlyHQ = $derived(user?.role === 'staff' && !user?.branch_id);
	let isCompanyManager = $derived(user?.role === 'company_manager');
	let canSeeBranchFilter = $derived((isCompanyManager || isReadonlyHQ) && branches.length > 0);

	// Branch filter state
	let selectedBranches = $state<string[]>([]);
	let showBranchDropdown = $state(false);

	function toggleBranchFilter(branchId: string) {
		if (selectedBranches.includes(branchId)) {
			selectedBranches = selectedBranches.filter((id) => id !== branchId);
		} else {
			selectedBranches = [...selectedBranches, branchId];
		}
	}

	function selectAllBranches() {
		if (Array.isArray(branches)) {
			selectedBranches = branches
				.map((b) =>
					typeof b === 'object' && b !== null && 'id' in b ? (b as { id?: string }).id : undefined
				)
				.filter((id): id is string => id !== undefined);
		}
	}

	function clearBranchFilter() {
		selectedBranches = [];
	}

	let selectedBranchesLabel = $derived(() => {
		if (selectedBranches.length === 0) return 'All Branches';
		if (selectedBranches.length === branches.length) return 'All Branches';
		if (selectedBranches.length === 1 && Array.isArray(branches)) {
			const branchId = selectedBranches[0];
			const branch = branches.find((b) => {
				if (typeof b === 'object' && b !== null && 'id' in b) {
					return (b as { id?: string }).id === branchId;
				}
				return false;
			});
			const name =
				typeof branch === 'object' && branch !== null && 'name' in branch
					? (branch as { name?: string }).name
					: undefined;
			return name || '1 Branch';
		}
		return `${selectedBranches.length} Branches`;
	});

	let logTypes = $state([
		{ id: 'all', label: 'All', checked: true },
		{ id: 'type1', label: 'Text', checked: true },
		{ id: 'type2', label: 'Checkbox', checked: true },
		{ id: 'type3', label: 'Temperature', checked: true },
		{ id: 'type4', label: 'Dropdown', checked: true }
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

	let reportGenerated = $state(false);
	let arrangeBy = $state<'date' | 'logType'>('date');
	let isLoading = $state(false);
	let error = $state<string | null>(null);
	let logEntries = $state<LogEntry[]>([]);
	let filteredEntries = $state<LogEntry[]>([]);
	let includeTemperatureGraphs = $state(false);
	let reportRuns = $state<ReportRun[]>([]);
	let isReportRunsLoading = $state(false);
	let reportRunsError = $state<string | null>(null);
	let deletingReportId = $state<string | null>(null);

	// Temperature graph data types
	type TemperatureDataPoint = {
		value: number;
		date: string;
		entryId: string;
		period: string;
	};

	type TemperatureGraphData = {
		templateName: string;
		fieldLabel: string;
		unit: string;
		dataPoints: TemperatureDataPoint[];
	};

	let temperatureGraphs = $state<TemperatureGraphData[]>([]);

	// Extract temperature graph data from filtered entries
	function extractTemperatureGraphData(entries: LogEntry[]): TemperatureGraphData[] {
		// Group entries by template name
		const templateGroups = new SvelteMap<string, LogEntry[]>();
		entries.forEach((entry) => {
			if (!templateGroups.has(entry.template_name)) {
				templateGroups.set(entry.template_name, []);
			}
			templateGroups.get(entry.template_name)!.push(entry);
		});

		const graphs: TemperatureGraphData[] = [];

		templateGroups.forEach((groupEntries, templateName) => {
			// Only process templates that have 2+ entries (need multiple data points for a graph)
			if (groupEntries.length < 2) return;

			// Find temperature fields in the template layout
			const firstEntry = groupEntries[0];
			if (!firstEntry.template_layout) return;

			firstEntry.template_layout.forEach((field: TemplateField, fieldIndex: number) => {
				if (field.field_type !== 'temperature') return;

				const fieldLabel = field.props?.text || `Temperature ${fieldIndex + 1}`;
				const unit = field.props?.unit || '°C';

				const dataPoints: TemperatureDataPoint[] = [];

				groupEntries.forEach((entry) => {
					const entryData =
						typeof entry.entry_data === 'string'
							? (() => {
									try {
										return JSON.parse(entry.entry_data);
									} catch {
										return {};
									}
								})()
							: typeof entry.entry_data === 'object'
								? entry.entry_data
								: {};

					if (!entryData || typeof entryData !== 'object') return;

					// Access field data by array index, which is how entry_data is keyed
					const fieldValue = (entryData as Record<string | number, unknown>)[fieldIndex];

					if (
						fieldValue !== undefined &&
						fieldValue !== null &&
						fieldValue !== '' &&
						typeof Number(fieldValue) === 'number' &&
						!isNaN(Number(fieldValue))
					) {
						dataPoints.push({
							value: Number(fieldValue),
							date: entry.created_at,
							entryId: entry.id,
							period: entry.period
						});
					}
				});

				// Sort by date ascending
				dataPoints.sort((a, b) => new Date(a.date).getTime() - new Date(b.date).getTime());

				if (dataPoints.length >= 2) {
					graphs.push({ templateName, fieldLabel, unit, dataPoints });
				}
			});
		});

		return graphs;
	}

	// SVG chart helper functions
	function buildChartPath(
		points: TemperatureDataPoint[],
		chartWidth: number,
		chartHeight: number,
		minVal: number,
		maxVal: number
	): string {
		const range = maxVal - minVal || 1;
		const padding = range * 0.1;
		const adjMin = minVal - padding;
		const adjMax = maxVal + padding;
		const adjRange = adjMax - adjMin;

		return points
			.map((p, i) => {
				const x = points.length === 1 ? chartWidth / 2 : (i / (points.length - 1)) * chartWidth;
				const y = chartHeight - ((p.value - adjMin) / adjRange) * chartHeight;
				return `${i === 0 ? 'M' : 'L'} ${x.toFixed(1)} ${y.toFixed(1)}`;
			})
			.join(' ');
	}

	function buildAreaPath(
		points: TemperatureDataPoint[],
		chartWidth: number,
		chartHeight: number,
		minVal: number,
		maxVal: number
	): string {
		const range = maxVal - minVal || 1;
		const padding = range * 0.1;
		const adjMin = minVal - padding;
		const adjMax = maxVal + padding;
		const adjRange = adjMax - adjMin;

		const linePath = points
			.map((p, i) => {
				const x = points.length === 1 ? chartWidth / 2 : (i / (points.length - 1)) * chartWidth;
				const y = chartHeight - ((p.value - adjMin) / adjRange) * chartHeight;
				return `${i === 0 ? 'M' : 'L'} ${x.toFixed(1)} ${y.toFixed(1)}`;
			})
			.join(' ');

		const lastX = points.length === 1 ? chartWidth / 2 : chartWidth;
		const firstX = points.length === 1 ? chartWidth / 2 : 0;

		return `${linePath} L ${lastX.toFixed(1)} ${chartHeight} L ${firstX.toFixed(1)} ${chartHeight} Z`;
	}

	function getChartPoints(
		points: TemperatureDataPoint[],
		chartWidth: number,
		chartHeight: number,
		minVal: number,
		maxVal: number
	): Array<{ x: number; y: number; value: number; date: string; period: string }> {
		const range = maxVal - minVal || 1;
		const padding = range * 0.1;
		const adjMin = minVal - padding;
		const adjMax = maxVal + padding;
		const adjRange = adjMax - adjMin;

		return points.map((p, i) => ({
			x: points.length === 1 ? chartWidth / 2 : (i / (points.length - 1)) * chartWidth,
			y: chartHeight - ((p.value - adjMin) / adjRange) * chartHeight,
			value: p.value,
			date: new Date(p.date).toLocaleDateString(),
			period: p.period
		}));
	}

	function getYAxisTicks(
		minVal: number,
		maxVal: number,
		chartHeight: number
	): Array<{ value: number; y: number }> {
		const range = maxVal - minVal || 1;
		const padding = range * 0.1;
		const adjMin = minVal - padding;
		const adjMax = maxVal + padding;
		const adjRange = adjMax - adjMin;
		const tickCount = 5;
		const ticks: Array<{ value: number; y: number }> = [];

		for (let i = 0; i <= tickCount; i++) {
			const value = adjMin + (i / tickCount) * adjRange;
			const y = chartHeight - (i / tickCount) * chartHeight;
			ticks.push({ value: Math.round(value * 10) / 10, y });
		}

		return ticks;
	}

	// Function to normalize field type (handle all text field variants as 'text')
	function normalizeFieldType(fieldType: unknown): string {
		const ft = fieldType as string | undefined | null;
		if (!ft || ft === 'input' || ft === 'text' || ft === 'text_input' || ft === 'label')
			return 'text';
		return ft;
	}

	// Function to parse entry data and extract readable values
	function parseEntryData(
		entryData: unknown,
		templateLayout: TemplateField[],
		excludeFieldTypes: string[] = []
	): string {
		if (!entryData) return 'No data available';

		try {
			// Parse the entry data if it's a string
			let data: unknown;
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
						.filter(([, value]) => value !== null && value !== undefined && value !== '')
						.map(([k, value]) => `${k}: ${value}`);
					return entries.length > 0 ? entries.join(', ') : 'No data entered';
				}
				return String(data);
			}

			const results: string[] = [];

			// Try different possible field identifier patterns
			templateLayout.forEach((field: TemplateField, index: number) => {
				// Skip fields of excluded types - handle null/undefined as text fields
				const fieldType = normalizeFieldType(field.field_type);
				if (excludeFieldTypes.includes(fieldType)) {
					return; // Skip this field
				}

				// Field data is keyed by array index in entry_data
				const fieldValue = (data as Record<string | number, unknown>)[index];

				if (fieldValue === undefined || fieldValue === null || fieldValue === '') {
					return; // Skip empty fields
				}

				// Format different field types
				let displayValue: unknown = fieldValue;

				if (fieldType === 'temperature' && typeof fieldValue === 'number') {
					displayValue = `${fieldValue}°C`;
				} else if (fieldType === 'checkbox' || fieldType === 'boolean') {
					displayValue = fieldValue ? 'Yes' : 'No';
				} else if (fieldType === 'dropdown' || fieldType === 'select') {
					// Try to find the option label
					const options = field.props?.options;
					if (Array.isArray(options) && typeof fieldValue === 'string') {
						const matchingOption = options.find((opt) =>
							typeof opt === 'string' ? opt === fieldValue : false
						);
						if (matchingOption) {
							displayValue = matchingOption;
						}
					}
				}

				// Get field label
				const fieldLabel = field.props?.text || `Field ${index + 1}`;
				results.push(`${fieldLabel}: ${displayValue}`);
			});

			// If no results from template parsing, try to show raw data
			if (results.length === 0 && typeof data === 'object' && data !== null) {
				const entries = Object.entries(data)
					.filter(([, value]) => value !== null && value !== undefined && value !== '')
					.map(([k, value]) => `${k}: ${value}`);
				return entries.length > 0 ? entries.join(', ') : 'No data entered';
			}

			return results.length > 0 ? results.join(', ') : 'No data entered';
		} catch {
			console.error('Error parsing entry data:', 'error');
			return 'Error parsing data';
		}
	}

	// Function to categorize log type based on template layout
	function categorizeLogType(templateLayout: TemplateField[]): string {
		if (!templateLayout || templateLayout.length === 0) return 'Text Logs';

		const fieldTypes = templateLayout.map((field) => field.field_type).filter(Boolean);

		if (fieldTypes.includes('temperature')) return 'Temperature Logs';
		if (fieldTypes.includes('checkbox')) return 'Checkbox Logs';
		if (fieldTypes.includes('dropdown')) return 'Dropdown Logs';

		return 'Text Logs';
	}

	// Function to get field type from a single field
	function getFieldType(field: Record<string, unknown>): string {
		const fieldType = field.field_type;
		if (fieldType === 'temperature') return 'Temperature Logs';
		if (fieldType === 'checkbox') return 'Checkbox Logs';
		if (fieldType === 'dropdown') return 'Dropdown Logs';
		return 'Text Logs';
	}

	// Function to extract individual components from entries
	function extractComponents(entries: LogEntry[], excludedFieldTypes: string[]): LogComponent[] {
		const components: LogComponent[] = [];

		entries.forEach((entry) => {
			if (!entry.template_layout) return;

			entry.template_layout.forEach((field: TemplateField, index: number) => {
				const fieldType = normalizeFieldType(field.field_type);

				// Skip excluded field types
				if (excludedFieldTypes.includes(fieldType)) return;

				// Parse the field data
				const fieldData = parseFieldData(entry.entry_data, field, index);

				// Only add if field has data
				if (fieldData && fieldData !== 'No data entered' && fieldData !== 'No data available') {
					components.push({
						entry,
						field,
						fieldType: getFieldType(field),
						fieldData,
						fieldIndex: index,
						componentId: `${entry.id}-field-${index}`
					});
				}
			});
		});

		return components;
	}

	// Function to parse individual field data
	function parseFieldData(entryData: unknown, field: TemplateField, fieldIndex: number): string {
		if (!entryData) return '';

		try {
			let data: unknown;
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

			const dataRecord = data as Record<string | number, unknown>;
			const fieldRecord = field as unknown as Record<string, unknown>;
			const propsRecord = (field.props ?? {}) as Record<string, unknown>;

			// First try index-keyed data, then fall back to common field identifiers.
			let fieldValue = dataRecord[fieldIndex];
			if (fieldValue === undefined || fieldValue === null || fieldValue === '') {
				const possibleIds = [
					fieldRecord.field_id,
					fieldRecord.id,
					fieldRecord.name,
					propsRecord.name,
					propsRecord.id,
					`field_${fieldIndex}`,
					fieldIndex.toString()
				].filter(Boolean) as (string | number)[];

				for (const id of possibleIds) {
					const value = dataRecord[id];
					if (value !== undefined && value !== null && value !== '') {
						fieldValue = value;
						break;
					}
				}
			}

			if (fieldValue === undefined || fieldValue === null || fieldValue === '') {
				return '';
			}

			// Format the field value
			const fieldType = field.field_type;
			let displayValue = fieldValue;

			if (fieldType === 'temperature' && typeof fieldValue === 'number') {
				displayValue = `${fieldValue}°C`;
			} else if (fieldType === 'checkbox' || fieldType === 'boolean') {
				displayValue = fieldValue ? 'Yes' : 'No';
			} else if (fieldType === 'dropdown' || fieldType === 'select') {
				const options = field.props?.options;
				if (Array.isArray(options) && typeof fieldValue === 'string') {
					// Options are simple strings, so just check if the value is in the list
					if (options.includes(fieldValue)) {
						displayValue = fieldValue;
					}
				}
			}

			const fieldLabel = field.props?.text || `Field ${fieldIndex + 1}`;
			return `${fieldLabel}: ${displayValue}`;
		} catch {
			return '';
		}
	}

	// Function to get excluded field types based on checkbox selections
	function getExcludedFieldTypes(): string[] {
		const excludedTypes: string[] = [];

		// Check which log types are NOT selected
		logTypes.forEach((type) => {
			if (type.id !== 'all' && !type.checked) {
				// Map log type ids to field types
				if (type.id === 'type3') {
					excludedTypes.push('temperature');
				} else if (type.id === 'type2') {
					excludedTypes.push('checkbox');
				} else if (type.id === 'type4') {
					excludedTypes.push('dropdown');
				} else if (type.id === 'type1') {
					// Text logs include fields with no field_type or undefined/null field_type
					excludedTypes.push('text');
				}
			}
		});

		return excludedTypes;
	}

	// Function to check if an entry has any remaining fields after filtering
	function hasRemainingFields(
		templateLayout: TemplateField[],
		excludeFieldTypes: string[]
	): boolean {
		if (!templateLayout || templateLayout.length === 0) return true;

		// Entry should remain visible if at least one field remains after log-type exclusions.
		return templateLayout.some((field) => {
			const fieldType = normalizeFieldType(field.field_type);
			return !excludeFieldTypes.includes(fieldType);
		});
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

	// Update ISO date when text input changes
	function updateDateFromText(value: string, isFrom: boolean) {
		const iso = formatToISO(value);
		if (iso) {
			if (isFrom) {
				dateFromISO = iso;
			} else {
				if (iso <= currentDateISO) {
					dateToISO = iso;
				}
			}
		}
	}

	function clampDateToTodayOnBlur() {
		const iso = formatToISO(dateTo);
		if (iso && iso > currentDateISO) {
			dateTo = currentDateFormatted;
			dateToISO = currentDateISO;
		}
	}

	function handleAllCheckboxChange(checked: boolean) {
		logTypes.forEach((logType) => {
			if (logType.id !== 'all') {
				logType.checked = checked;
			}
		});

		// Re-generate report if one was already generated
		if (reportGenerated && logEntries.length > 0) {
			refilterReport();
		}
	}

	function handleIndividualCheckboxChange() {
		const allCheckbox = logTypes.find((logType) => logType.id === 'all');
		const otherCheckboxes = logTypes.filter((logType) => logType.id !== 'all');
		const allOthersChecked = otherCheckboxes.every((logType) => logType.checked);

		if (allCheckbox) {
			allCheckbox.checked = allOthersChecked;
		}

		// Re-generate report if one was already generated
		if (reportGenerated && logEntries.length > 0) {
			refilterReport();
		}
	}

	function refilterReport() {
		// Re-apply filtering with current excluded types
		const excludedFieldTypes = getExcludedFieldTypes();
		filteredEntries = logEntries.filter((entry) => {
			// Filter by date range
			const entryDate = new SvelteDate(entry.created_at);
			const fromDate = new SvelteDate(dateFromISO);
			const toDate = new SvelteDate(dateToISO);
			toDate.setHours(23, 59, 59, 999);
			const isInDateRange = entryDate >= fromDate && entryDate <= toDate;

			// Check if entry has any remaining fields after filtering
			const hasFields = hasRemainingFields(entry.template_layout, excludedFieldTypes);

			return isInDateRange && hasFields;
		});

		// Sort entries based on arrange preference
		if (arrangeBy === 'date') {
			filteredEntries.sort(
				(a, b) => new SvelteDate(b.created_at).getTime() - new SvelteDate(a.created_at).getTime()
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

		// Re-extract temperature graphs if enabled
		if (includeTemperatureGraphs) {
			temperatureGraphs = extractTemperatureGraphData(filteredEntries);
		}
	}

	function getSelectedLogTypeIds(): string[] {
		return logTypes.filter((type) => type.id !== 'all' && type.checked).map((type) => type.id);
	}

	function applySelectedLogTypeIds(selectedIds: string[]) {
		const selectedSet = new SvelteSet(selectedIds);
		logTypes = logTypes.map((type) => {
			if (type.id === 'all') {
				return { ...type, checked: false };
			}
			return { ...type, checked: selectedSet.has(type.id) };
		});
		handleIndividualCheckboxChange();
	}

	function buildCurrentReportParams(): SavedReportParams {
		const selectedBranchIds = [...selectedBranches].sort();
		const selectedLogTypeIds = getSelectedLogTypeIds().sort();
		return {
			date_from_iso: dateFromISO,
			date_to_iso: dateToISO,
			selected_branch_ids: selectedBranchIds,
			selected_log_type_ids: selectedLogTypeIds,
			arrange_by: arrangeBy,
			include_temperature_graphs: includeTemperatureGraphs,
			params_version: 1
		};
	}

	function applyReportParams(params: SavedReportParams) {
		dateFromISO = params.date_from_iso;
		dateToISO = params.date_to_iso > currentDateISO ? currentDateISO : params.date_to_iso;
		dateFrom = formatFromISO(params.date_from_iso) || dateFrom;
		dateTo = formatFromISO(dateToISO) || dateTo;
		selectedBranches = [...params.selected_branch_ids];
		arrangeBy = params.arrange_by;
		includeTemperatureGraphs = params.include_temperature_graphs;
		applySelectedLogTypeIds(params.selected_log_type_ids);
	}

	function formatFromISO(iso: string): string {
		const parts = iso.split('-');
		if (parts.length !== 3) return '';
		const [year, month, day] = parts;
		return `${day}/${month}/${year}`;
	}

	async function loadReportRuns() {
		isReportRunsLoading = true;
		reportRunsError = null;
		try {
			const res = await fetch('/api/reports/runs?limit=20');
			if (!res.ok) {
				throw new Error('Failed to load saved reports');
			}
			const data = (await res.json()) as { report_runs?: ReportRun[] };
			reportRuns = data.report_runs ?? [];
		} catch (err) {
			reportRunsError = err instanceof Error ? err.message : 'Failed to load saved reports';
		} finally {
			isReportRunsLoading = false;
		}
	}

	async function saveReportRun() {
		const payload = {
			params: buildCurrentReportParams()
		};

		const res = await fetch('/api/reports/runs', {
			method: 'POST',
			headers: { 'content-type': 'application/json' },
			body: JSON.stringify(payload)
		});

		if (!res.ok) {
			throw new Error('Failed to save report settings');
		}
	}

	async function runSavedReport(run: ReportRun) {
		applyReportParams(run.params);

		await generateReport();
		await loadReportRuns();
	}

	async function deleteReportRun(reportId: string) {
		deletingReportId = reportId;
		reportRunsError = null;
		try {
			const res = await fetch(`/api/reports/runs/${reportId}`, {
				method: 'DELETE'
			});
			if (!res.ok) {
				let errorMessage = 'Failed to delete saved report';
				try {
					const data = (await res.json()) as { error?: string };
					if (data.error) {
						errorMessage = `${errorMessage} (${res.status}): ${data.error}`;
					} else {
						errorMessage = `${errorMessage} (${res.status})`;
					}
				} catch {
					errorMessage = `${errorMessage} (${res.status})`;
				}

				console.error('Delete report failed', { reportId, status: res.status });
				throw new Error(errorMessage);
			}
			await loadReportRuns();
		} catch (err) {
			console.error('Delete report exception', { reportId, err });
			reportRunsError = err instanceof Error ? err.message : 'Failed to delete saved report';
		} finally {
			deletingReportId = null;
		}
	}

	onMount(async () => {
		await loadReportRuns();
	});

	async function generateReport() {
		isLoading = true;
		error = null;
		reportGenerated = false;

		try {
			// Build query params for branch filter
			let branchIds = '';
			if (
				(isCompanyManager || isReadonlyHQ) &&
				selectedBranches.length > 0 &&
				selectedBranches.length < branches.length
			) {
				branchIds = selectedBranches.join(',');
			}

			// Fetch log entries - use admin endpoint for readonly HQ to get all company logs
			const response =
				isReadonlyHQ || isCompanyManager
					? await api.GET(`/logs/admin/entries`, { params: { query: { branch_ids: branchIds } } })
					: await api.GET('/logs/entries');

			if (!response.data) {
				throw new Error('Failed to fetch log entries');
			}

			logEntries = response.data.entries;

			// Collect available log types from actual data
			const logTypesSet = new SvelteSet<string>();
			logEntries.forEach((entry) => {
				const category = categorizeLogType(entry.template_layout);
				logTypesSet.add(category);
			});

			// Filter entries based on date range and selected log types
			const selectedLogTypes = logTypes
				.filter((type) => type.checked && type.id !== 'all')
				.map((type) => type.label);

			// If "All" is selected or no specific types selected, show all types
			// eslint-disable-next-line @typescript-eslint/no-unused-vars
			const _showAllTypes =
				logTypes.find((type) => type.id === 'all')?.checked || selectedLogTypes.length === 0;

			// Get excluded field types for filtering
			const excludedFieldTypes = getExcludedFieldTypes();

			filteredEntries = logEntries.filter((entry) => {
				// Filter by date range
				const entryDate = new SvelteDate(entry.created_at);
				const fromDate = new SvelteDate(dateFromISO);
				const toDate = new SvelteDate(dateToISO);
				// Set time to end of day for 'to' date
				toDate.setHours(23, 59, 59, 999);

				const isInDateRange = entryDate >= fromDate && entryDate <= toDate;

				// Check if entry has any remaining fields after filtering
				const hasFields = hasRemainingFields(entry.template_layout, excludedFieldTypes);

				return isInDateRange && hasFields;
			});

			// Sort entries based on arrange preference
			if (arrangeBy === 'date') {
				filteredEntries.sort(
					(a, b) => new SvelteDate(b.created_at).getTime() - new SvelteDate(a.created_at).getTime()
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

			// Extract temperature graph data if enabled
			if (includeTemperatureGraphs) {
				temperatureGraphs = extractTemperatureGraphData(filteredEntries);
			} else {
				temperatureGraphs = [];
			}

			try {
				await saveReportRun();
				await loadReportRuns();
			} catch {
				// Saving report run should never block report generation success.
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
			printWindow.document.write(`<html>
<head>
<title>Log Report</title>
<style>${PDF_STYLES.report}</style>
</head>
<body>
${reportContent}
</body>
</html>`);
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
		return `<!DOCTYPE html>
<html xmlns:o="urn:schemas-microsoft-com:office:office" xmlns:w="urn:schemas-microsoft-com:office:word" xmlns="http://www.w3.org/TR/REC-html40">
<head>
	<meta charset="utf-8">
	<meta name="ProgId" content="Word.Document">
	<meta name="Generator" content="Microsoft Word">
	<meta name="Originator" content="Microsoft Word">
	<title>Log Report</title>
	<style>${PDF_STYLES.word}</style>
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
			const excludedFieldTypes = getExcludedFieldTypes();
			const components = extractComponents(filteredEntries, excludedFieldTypes);
			const groupedComponents = components.reduce(
				(acc: Record<string, typeof components>, component) => {
					if (!acc[component.fieldType]) acc[component.fieldType] = [];
					acc[component.fieldType].push(component);
					return acc;
				},
				{}
			);

			Object.entries(groupedComponents).forEach(([fieldType, componentGroup]) => {
				content += `<h2>${fieldType} (${componentGroup.length} components)</h2>`;
				componentGroup.forEach((component) => {
					content += generateWordComponentHTML(component);
				});
			});
		} else {
			filteredEntries.forEach((entry) => {
				content += generateWordEntryHTML(entry);
			});
		}

		return content;
	}

	function generateWordComponentHTML(component: LogComponent): string {
		const statusBg =
			component.entry.status === 'submitted' ? 'var(--button-primary)' : 'var(--orange)';
		const entryTitle = formatTemplateName(component.entry.template_name, component.entry.period);

		return `
		<div class="entry-box" style="page-break-inside: avoid; page-break-after: auto;">
			<p class="entry-title" style="page-break-after: avoid; keep-with-next: always;">${entryTitle}</p>
			<p class="entry-id" style="page-break-after: avoid; keep-with-next: always;">ID: ${component.entry.id.slice(0, 8)}...</p>
			<p class="status-badge" style="background-color: ${statusBg}; page-break-after: avoid; keep-with-next: always;">${component.entry.status}</p>
			
			<div class="entry-data-box" style="page-break-inside: avoid; keep-with-next: always;">
				<span class="field-label">Component Data:</span> ${component.fieldData}
			</div>
			
			<p class="field-row" style="page-break-after: avoid; keep-with-next: always;">
				<span class="field-label">Created:</span> ${new Date(component.entry.created_at).toLocaleString()}
			</p>
			
			${component.entry.submitted_at ? `<p class="field-row" style="page-break-after: avoid; keep-with-next: always;"><span class="field-label">Submitted:</span> ${new Date(component.entry.submitted_at).toLocaleString()}</p>` : ''}
			
			<p class="field-row" style="keep-with-next: avoid;">
				<span class="field-label">Period:</span> ${component.entry.period}
			</p>
		</div>
		<p style="page-break-after: auto; margin-bottom: 0;">&nbsp;</p>
		`;
	}

	function generateWordEntryHTML(entry: LogEntry): string {
		const excludedFieldTypes = getExcludedFieldTypes();
		const entryData = parseEntryData(entry.entry_data, entry.template_layout, excludedFieldTypes);
		const statusBg = entry.status === 'submitted' ? 'var(--button-primary)' : 'var(--orange)';
		const entryTitle = formatTemplateName(entry.template_name, entry.period);

		// Use paragraphs with Word-specific keep-together formatting
		return `
		<div class="entry-box" style="page-break-inside: avoid; page-break-after: auto;">
			<p class="entry-title" style="page-break-after: avoid; keep-with-next: always;">${entryTitle}</p>
			<p class="entry-id" style="page-break-after: avoid; keep-with-next: always;">ID: ${entry.id.slice(0, 8)}...</p>
			<p class="status-badge" style="background-color: ${statusBg}; page-break-after: avoid; keep-with-next: always;">${entry.status}</p>
			
			<div class="entry-data-box" style="page-break-inside: avoid; keep-with-next: always;">
				<span class="field-label">Entry Data:</span> ${entryData}
			</div>
			
			<p class="field-row" style="page-break-after: avoid; keep-with-next: always;">
				<span class="field-label">Created:</span> ${new Date(entry.created_at).toLocaleString()}
			</p>
			
			${entry.submitted_at ? `<p class="field-row" style="page-break-after: avoid; keep-with-next: always;"><span class="field-label">Submitted:</span> ${new Date(entry.submitted_at).toLocaleString()}</p>` : ''}
			
			<p class="field-row" style="keep-with-next: avoid;">
				<span class="field-label">Period:</span> ${entry.period}
			</p>
		</div>
		<p style="page-break-after: auto; margin-bottom: 0;">&nbsp;</p>
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
			const excludedFieldTypes = getExcludedFieldTypes();
			const components = extractComponents(filteredEntries, excludedFieldTypes);
			const groupedComponents = components.reduce(
				(acc: Record<string, typeof components>, component) => {
					if (!acc[component.fieldType]) acc[component.fieldType] = [];
					acc[component.fieldType].push(component);
					return acc;
				},
				{}
			);

			Object.entries(groupedComponents).forEach(([fieldType, componentGroup]) => {
				rtfContent += `\\b\\fs26 ${fieldType} (${componentGroup.length} components)\\b0\\fs24\\par\\par`;
				componentGroup.forEach((component) => {
					rtfContent += generateRTFComponent(component);
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

	function generateRTFComponent(component: LogComponent): string {
		const statusColor = component.entry.status === 'submitted' ? '\\cf2' : '\\cf3';
		const entryTitle = formatTemplateName(component.entry.template_name, component.entry.period);

		let rtf = `\\pard\\box\\brdrs\\brdrw10\\brdrcf1\\par`;
		rtf += `\\b ${entryTitle}\\b0\\par`;
		rtf += `\\i ID: ${component.entry.id.slice(0, 8)}...\\i0\\par`;
		rtf += `${statusColor}\\b ${component.entry.status}\\b0\\cf1\\par\\par`;
		rtf += `\\b Component Data:\\b0 ${component.fieldData}\\par\\par`;
		rtf += `\\b Created:\\b0 ${new Date(component.entry.created_at).toLocaleString()}\\par`;
		if (component.entry.submitted_at) {
			rtf += `\\b Submitted:\\b0 ${new Date(component.entry.submitted_at).toLocaleString()}\\par`;
		}
		rtf += `\\b Period:\\b0 ${component.entry.period}\\par`;
		rtf += `\\par\\pard\\par`;

		return rtf;
	}

	function generateRTFEntry(entry: LogEntry): string {
		const excludedFieldTypes = getExcludedFieldTypes();
		const entryData = parseEntryData(entry.entry_data, entry.template_layout, excludedFieldTypes);
		const statusColor = entry.status === 'submitted' ? '\\cf2' : '\\cf3';
		const entryTitle = formatTemplateName(entry.template_name, entry.period);

		let rtf = `\\pard\\box\\brdrs\\brdrw10\\brdrcf1\\par`;
		rtf += `\\b ${entryTitle}\\b0\\par`;
		rtf += `\\i ID: ${entry.id.slice(0, 8)}...\\i0\\par`;
		rtf += `${statusColor}\\b ${entry.status}\\b0\\cf1\\par\\par`;
		rtf += `\\b Entry Data:\\b0 ${entryData}\\par\\par`;
		rtf += `\\b Created:\\b0 ${new Date(entry.created_at).toLocaleString()}\\par`;
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
			const excludedFieldTypes = getExcludedFieldTypes();
			const components = extractComponents(filteredEntries, excludedFieldTypes);
			const groupedComponents = components.reduce(
				(acc: Record<string, typeof components>, component) => {
					if (!acc[component.fieldType]) acc[component.fieldType] = [];
					acc[component.fieldType].push(component);
					return acc;
				},
				{}
			);

			Object.entries(groupedComponents).forEach(([fieldType, componentGroup]) => {
				content += `<div class="group-header">${fieldType} (${componentGroup.length} components)</div>`;
				componentGroup.forEach((component) => {
					content += generateComponentHTML(component);
				});
			});
		} else {
			filteredEntries.forEach((entry) => {
				content += generateEntryHTML(entry);
			});
		}

		return content;
	}

	// Helper to format template name with period
	function formatTemplateName(templateName: string, period?: string): string {
		if (period && templateName.includes('{period}')) {
			return templateName.replace('{period}', period);
		}
		return templateName;
	}

	function generateComponentHTML(component: LogComponent): string {
		const entryTitle = formatTemplateName(component.entry.template_name, component.entry.period);
		return `
			<div class="entry">
				<div style="display: flex; justify-content: space-between; align-items: start; margin-bottom: 10px;">
					<div>
						<strong>${entryTitle}</strong>
						<small style="color: var(--grey-dark); margin-left: 10px;">ID: ${component.entry.id.slice(0, 8)}...</small>
					</div>
					<span class="status ${component.entry.status === 'submitted' ? 'submitted' : 'draft'}">${component.entry.status}</span>
				</div>
				<div class="entry-data">
					<strong>Component Data:</strong> ${component.fieldData}
				</div>
				<p><strong>Created:</strong> ${new Date(component.entry.created_at).toLocaleString()}</p>
				${component.entry.submitted_at ? `<p><strong>Submitted:</strong> ${new Date(component.entry.submitted_at).toLocaleString()}</p>` : ''}
				<p><strong>Period:</strong> ${component.entry.period}</p>
			</div>
		`;
	}

	function generateEntryHTML(entry: LogEntry): string {
		const excludedFieldTypes = getExcludedFieldTypes();
		const entryData = parseEntryData(entry.entry_data, entry.template_layout, excludedFieldTypes);
		const entryTitle = formatTemplateName(entry.template_name, entry.period);
		return `
			<div class="entry">
				<div style="display: flex; justify-content: space-between; align-items: start; margin-bottom: 10px;">
					<div>
						<strong>${entryTitle}</strong>
						<small style="color: var(--grey-dark); margin-left: 10px;">ID: ${entry.id.slice(0, 8)}...</small>
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
<div class="reports-page min-h-full" style="background-color: var(--bg-secondary);">
	<!-- Main Content -->
	<div class="mx-auto max-w-7xl px-6 py-6 lg:py-4">
		<h1
			class="mb-8 text-center text-3xl font-bold md:text-4xl lg:mb-5"
			style="color: var(--text-primary);"
		>
			Generate Report
		</h1>

		<div class="flex flex-col gap-8 lg:flex-row lg:gap-6">
			<!-- Left Side - Form -->
			<div class="w-full lg:w-96">
				<!-- Date From -->
				<div class="mb-8 lg:mb-5">
					<label
						for="date-from"
						class="mb-3 block text-lg font-bold"
						style="color: var(--text-primary);">Date From:</label
					>
					<DatePicker
						inputId="date-from"
						bind:value={dateFrom}
						onValueInput={(nextValue: string) => updateDateFromText(nextValue, true)}
						openCalendarAriaLabel="Open calendar for start date"
						pickerStyle="border-color: var(--border-primary); background-color: var(--bg-primary); min-width: 280px; sm:min-width: 320px; overflow: hidden; right: auto;"
					/>
				</div>

				<!-- Date To -->
				<div class="mb-8 lg:mb-5">
					<label
						for="date-to"
						class="mb-3 block text-lg font-bold"
						style="color: var(--text-primary);">Date To:</label
					>
					<DatePicker
						inputId="date-to"
						bind:value={dateTo}
						onValueInput={(nextValue: string) => updateDateFromText(nextValue, false)}
						onValueBlur={clampDateToTodayOnBlur}
						maxDate={today}
						openCalendarAriaLabel="Open calendar for end date"
						pickerStyle="border-color: var(--border-primary); background-color: var(--bg-primary); min-width: 280px; sm:min-width: 320px; overflow: hidden; right: auto;"
					/>
				</div>

				<!-- Log Types -->
				<div class="mb-8 lg:mb-5">
					<fieldset>
						<legend class="mb-3 block text-lg font-bold" style="color: var(--text-primary);"
							>Log Types:</legend
						>
						<div class="space-y-2">
							{#each logTypes.filter((logType) => logType.id === 'all') as logType (logType.id)}
								<button
									type="button"
									aria-pressed={logType.checked}
									onclick={() => {
										if (logType.id === 'all') {
											const nextChecked = !logType.checked;
											logType.checked = nextChecked;
											handleAllCheckboxChange(nextChecked);
										} else {
											logType.checked = !logType.checked;
											handleIndividualCheckboxChange();
										}
									}}
									class="w-full border-2 px-4 py-1.5 text-center text-xs font-semibold transition-all duration-150 hover:-translate-y-0.5"
									style={logType.checked
										? 'border-color: var(--button-primary); background-color: var(--button-primary); color: var(--button-text); box-shadow: 0 0 6px rgba(61, 122, 130, 0.25);'
										: 'border-color: var(--border-primary); background-color: var(--bg-primary); color: var(--text-primary);'}
								>
									{logType.label}
								</button>
							{/each}
							<div class="grid grid-cols-4 gap-2">
								{#each logTypes.filter((logType) => logType.id !== 'all') as logType (logType.id)}
									<button
										type="button"
										aria-pressed={logType.checked}
										onclick={() => {
											logType.checked = !logType.checked;
											handleIndividualCheckboxChange();
										}}
										class="w-full border-2 px-1.5 py-2 text-center text-[11px] leading-tight font-medium whitespace-nowrap transition-all duration-150 hover:-translate-y-0.5"
										style={logType.checked
											? 'border-color: var(--button-primary); background-color: var(--button-primary); color: var(--bg-primary); box-shadow: 0 0 6px rgba(61, 122, 130, 0.25);'
											: 'border-color: var(--border-primary); background-color: var(--bg-primary); color: var(--text-primary);'}
									>
										{logType.label}
									</button>
								{/each}
							</div>
						</div>
					</fieldset>
				</div>

				<!-- Branch Filter (for company managers and HQ, when branches exist) -->
				{#if canSeeBranchFilter}
					<div class="branch-filter-container mb-8 lg:mb-5" style="position: relative;">
						<legend class="mb-3 block text-lg font-bold" style="color: var(--text-primary);"
							>Branches:</legend
						>
						<div class="relative">
							<button
								type="button"
								onclick={() => (showBranchDropdown = !showBranchDropdown)}
								class="flex w-full items-center justify-between border-2 px-4 py-2"
								style="border-color: var(--border-primary); background-color: var(--bg-primary); color: var(--text-primary);"
							>
								<span>{selectedBranchesLabel()}</span>
								<svg
									width="16"
									height="16"
									viewBox="0 0 16 16"
									fill="none"
									stroke="currentColor"
									stroke-width="2"
								>
									<polyline points="4 6 8 10 12 6"></polyline>
								</svg>
							</button>
							{#if showBranchDropdown}
								<div
									class="absolute top-full left-0 z-50 mt-1 max-h-60 w-full overflow-auto rounded-md border-2 shadow-lg"
									style="border-color: var(--border-primary); background-color: var(--bg-primary);"
								>
									<button
										type="button"
										class="w-full px-4 py-2 text-left font-semibold hover:opacity-80"
										style="color: var(--text-primary);"
										onclick={() => {
											selectAllBranches();
											showBranchDropdown = false;
										}}
									>
										All Branches
									</button>
									<button
										type="button"
										class="w-full px-4 py-2 text-left hover:opacity-80"
										style="color: var(--text-primary);"
										onclick={() => {
											clearBranchFilter();
											showBranchDropdown = false;
										}}
									>
										Clear Selection
									</button>
									{#each branches as branch (branch.id)}
										<button
											type="button"
											class="flex w-full items-center gap-2 px-4 py-2 text-left hover:opacity-80"
											style="color: var(--text-primary);"
											onclick={() => toggleBranchFilter(branch.id)}
										>
											<input
												type="checkbox"
												checked={selectedBranches.includes(branch.id)}
												class="h-4 w-4"
											/>
											{branch.name}
										</button>
									{/each}
								</div>
							{/if}
						</div>
						{#if selectedBranches.length > 0 && selectedBranches.length < branches.length}
							<p class="mt-1 text-xs" style="color: var(--text-secondary);">
								Filtering by {selectedBranches.length} of {branches.length} branches
							</p>
						{/if}
					</div>
				{/if}

				<!-- Arrange By Options -->
				<div class="mb-8 lg:mb-5">
					<legend class="mb-3 block text-lg font-bold" style="color: var(--text-primary);"
						>Arrange By:</legend
					>
					<div class="flex gap-3">
						<button
							type="button"
							onclick={() => (arrangeBy = 'date')}
							class="flex-1 transform border-2 px-4 py-1.5 text-sm font-semibold transition-all duration-200 hover:scale-105 hover:shadow-md"
							style={arrangeBy === 'date'
								? 'border-color: var(--button-primary); background-color: var(--button-primary); color: var(--bg-primary); box-shadow: 0 0 8px rgba(61, 122, 130, 0.3);'
								: 'border-color: var(--border-primary); background-color: transparent; color: var(--text-secondary);'}
						>
							Date
						</button>
						<button
							type="button"
							onclick={() => (arrangeBy = 'logType')}
							class="flex-1 transform border-2 px-4 py-1.5 text-sm font-semibold transition-all duration-200 hover:scale-105 hover:shadow-md"
							style={arrangeBy === 'logType'
								? 'border-color: var(--button-primary); background-color: var(--button-primary); color: var(--bg-primary); box-shadow: 0 0 8px rgba(61, 122, 130, 0.3);'
								: 'border-color: var(--border-primary); background-color: transparent; color: var(--text-secondary);'}
						>
							Log Type
						</button>
					</div>
				</div>

				<!-- Temperature Graphs Toggle -->
				<button
					type="button"
					aria-pressed={includeTemperatureGraphs}
					onclick={() => {
						includeTemperatureGraphs = !includeTemperatureGraphs;
						if (reportGenerated) {
							if (includeTemperatureGraphs) {
								temperatureGraphs = extractTemperatureGraphData(filteredEntries);
							} else {
								temperatureGraphs = [];
							}
						}
					}}
					class="mb-8 w-full border-2 px-4 py-3 text-left transition-all duration-150 hover:-translate-y-0.5 lg:mb-5"
					style={includeTemperatureGraphs
						? 'border-color: var(--button-primary); background-color: var(--button-primary); color: var(--bg-primary); box-shadow: 0 0 8px rgba(61, 122, 130, 0.3);'
						: 'border-color: var(--border-primary); background-color: var(--bg-primary); color: var(--text-primary);'}
				>
					<div class="flex min-w-0 items-start justify-between gap-3 sm:items-center">
						<div class="min-w-0">
							<p class="text-base font-semibold">Temperature Graphs</p>
							<p
								class="mt-1 text-[11px] wrap-break-word sm:whitespace-nowrap"
								style={includeTemperatureGraphs
									? 'color: rgba(255, 255, 255, 0.9);'
									: 'color: var(--text-secondary);'}
							>
								Line graphs for temperature fields (requires 2+ entries)
							</p>
						</div>
						<span
							class="min-w-11 shrink-0 text-right text-xs font-semibold tracking-wide"
							style={includeTemperatureGraphs
								? 'color: var(--bg-primary);'
								: 'color: var(--text-secondary);'}
						>
							{includeTemperatureGraphs ? 'ON' : 'OFF'}
						</span>
					</div>
				</button>

				<!-- Generate Button -->
				<div class="flex justify-center">
					<button
						onclick={generateReport}
						class="flex transform items-center gap-2 border-2 px-8 py-2 font-medium transition-all duration-200 hover:scale-105 hover:opacity-90 hover:shadow-md"
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

				<!-- Saved Report Runs -->
				<div class="mt-8 lg:mt-5">
					<h3 class="mb-3 text-lg font-bold" style="color: var(--text-primary);">Recent Reports</h3>
					{#if reportRunsError}
						<p class="mb-2 text-sm text-red-500">{reportRunsError}</p>
					{/if}
					{#if isReportRunsLoading}
						<p class="text-sm" style="color: var(--text-secondary);">Loading saved reports...</p>
					{:else if reportRuns.length === 0}
						<p class="text-sm" style="color: var(--text-secondary);">No saved reports yet.</p>
					{:else}
						<div class="space-y-2 lg:max-h-58 lg:overflow-y-auto lg:pr-1">
							{#each reportRuns as run (run.id)}
								<div
									class="w-full cursor-pointer rounded border-2 px-3 py-1.5 transition-all duration-150 hover:opacity-95 hover:shadow-md"
									style="border-color: var(--border-primary); background-color: var(--bg-primary); color: var(--text-primary);"
									role="button"
									tabindex="0"
									onclick={() => runSavedReport(run)}
									onkeydown={(e) => {
										if (e.key === 'Enter' || e.key === ' ') {
											e.preventDefault();
											runSavedReport(run);
										}
									}}
								>
									<div class="flex items-center justify-between gap-2">
										<div class="text-sm font-semibold">
											{formatFromISO(run.params.date_from_iso)} - {formatFromISO(
												run.params.date_to_iso
											)}
										</div>
										<button
											type="button"
											disabled={deletingReportId === run.id}
											onclick={(e) => {
												e.stopPropagation();
												deleteReportRun(run.id);
											}}
											class="cursor-pointer rounded border px-2 py-0.5 text-xs transition-colors hover:bg-red-50 hover:text-red-700 disabled:cursor-not-allowed disabled:opacity-60"
											style="border-color: var(--button-secondary); color: var(--button-secondary);"
										>
											{deletingReportId === run.id ? 'Deleting...' : 'Delete'}
										</button>
									</div>
									<div class="text-xs" style="color: var(--text-secondary);">
										Used {run.use_count} time(s) • {new Date(run.last_used_at).toLocaleString()}
									</div>
								</div>
							{/each}
						</div>
					{/if}
				</div>
			</div>

			<!-- Right Side - Report Preview -->
			<div class="w-full lg:flex-1">
				<!-- Download Buttons -->
				<div class="mb-4 flex flex-col gap-2 sm:flex-row sm:justify-end sm:gap-4">
					<button
						onclick={exportToPDF}
						disabled={!reportGenerated || filteredEntries.length === 0}
						class="transform border-2 px-4 py-2 text-sm font-medium transition-all duration-200 hover:scale-105 hover:border-blue-400 hover:bg-blue-50 hover:text-blue-700 hover:shadow-lg disabled:cursor-not-allowed disabled:opacity-50 disabled:hover:transform-none disabled:hover:shadow-none sm:text-base"
						style="border-color: var(--border-primary); color: var(--text-primary); background-color: var(--bg-primary);"
					>
						Download PDF
					</button>
					<button
						onclick={() => exportToWord('docx')}
						disabled={!reportGenerated || filteredEntries.length === 0}
						class="transform border-2 px-4 py-2 text-sm font-medium transition-all duration-200 hover:scale-105 hover:border-green-400 hover:bg-green-50 hover:text-green-700 hover:shadow-lg disabled:cursor-not-allowed disabled:opacity-50 disabled:hover:transform-none disabled:hover:shadow-none sm:text-base"
						style="border-color: var(--border-primary); color: var(--text-primary); background-color: var(--bg-primary);"
					>
						Download DOCX
					</button>
					<button
						onclick={() => exportToWord('rtf')}
						disabled={!reportGenerated || filteredEntries.length === 0}
						class="transform border-2 px-4 py-2 text-sm font-medium transition-all duration-200 hover:scale-105 hover:border-purple-400 hover:bg-purple-50 hover:text-purple-700 hover:shadow-lg disabled:cursor-not-allowed disabled:opacity-50 disabled:hover:transform-none disabled:hover:shadow-none sm:text-base"
						style="border-color: var(--border-primary); color: var(--text-primary); background-color: var(--bg-primary);"
					>
						Download RTF
					</button>
				</div>

				<!-- Report Preview Area -->
				<div
					class="min-h-104 border-2 p-4 sm:min-h-120 sm:p-6 lg:min-h-108 lg:p-5"
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

							<!-- Temperature Graphs (shown above entries when enabled) -->
							{#if includeTemperatureGraphs && temperatureGraphs.length > 0}
								<div class="mb-8">
									<h3
										class="mb-4 border-b-2 pb-2 text-lg font-bold"
										style="color: var(--text-primary); border-color: var(--border-primary);"
									>
										📈 Temperature Graphs
									</h3>
									{#each temperatureGraphs as graph (graph.dataPoints)}
										{@const values = graph.dataPoints.map((p) => p.value)}
										{@const minVal = Math.min(...values)}
										{@const maxVal = Math.max(...values)}
										{@const avgVal =
											Math.round((values.reduce((a, b) => a + b, 0) / values.length) * 10) / 10}
										{@const chartWidth = 500}
										{@const chartHeight = 200}
										{@const chartPoints = getChartPoints(
											graph.dataPoints,
											chartWidth,
											chartHeight,
											minVal,
											maxVal
										)}
										{@const yTicks = getYAxisTicks(minVal, maxVal, chartHeight)}
										<div
											class="mb-6 rounded-lg border p-4"
											style="border-color: var(--border-primary); background-color: var(--bg-secondary);"
										>
											<div class="mb-3 flex flex-wrap items-center justify-between gap-2">
												<div>
													<h4 class="font-bold" style="color: var(--text-primary);">
														{graph.templateName}
													</h4>
													<p class="text-sm" style="color: var(--text-secondary);">
														{graph.fieldLabel} ({graph.unit})
													</p>
												</div>
												<div
													class="flex flex-wrap gap-4 text-xs"
													style="color: var(--text-secondary);"
												>
													<span>Min: <strong>{minVal}{graph.unit}</strong></span>
													<span>Max: <strong>{maxVal}{graph.unit}</strong></span>
													<span>Avg: <strong>{avgVal}{graph.unit}</strong></span>
													<span>Entries: <strong>{graph.dataPoints.length}</strong></span>
												</div>
											</div>
											<div
												class="overflow-x-auto rounded"
												style="background-color: var(--bg-primary);"
											>
												<svg
													viewBox="-60 -10 {chartWidth + 80} {chartHeight + 60}"
													class="w-full"
													style="min-width: 400px; max-height: 300px;"
													xmlns="http://www.w3.org/2000/svg"
												>
													<!-- Grid lines -->
													{#each yTicks as tick (tick.value)}
														<line
															x1="0"
															y1={tick.y}
															x2={chartWidth}
															y2={tick.y}
															stroke="var(--border-primary)"
															stroke-width="0.5"
															stroke-dasharray="4,4"
															opacity="0.5"
														/>
														<text
															x="-8"
															y={tick.y + 4}
															text-anchor="end"
															fill="var(--text-secondary)"
															font-size="11">{tick.value}</text
														>
													{/each}

													<!-- Area fill -->
													<path
														d={buildAreaPath(
															graph.dataPoints,
															chartWidth,
															chartHeight,
															minVal,
															maxVal
														)}
														fill="var(--button-primary)"
														opacity="0.1"
													/>

													<!-- Line -->
													<path
														d={buildChartPath(
															graph.dataPoints,
															chartWidth,
															chartHeight,
															minVal,
															maxVal
														)}
														fill="none"
														stroke="var(--button-primary)"
														stroke-width="2.5"
														stroke-linejoin="round"
														stroke-linecap="round"
													/>

													<!-- Data points -->
													{#each chartPoints as point (point.x + '-' + point.y)}
														<g class="chart-point">
															<circle
																cx={point.x}
																cy={point.y}
																r="4"
																fill="var(--button-primary)"
																stroke="var(--bg-primary)"
																stroke-width="2"
															/>
															<!-- Hover target -->
															<circle
																cx={point.x}
																cy={point.y}
																r="12"
																fill="transparent"
																class="cursor-pointer"
															/>
															<!-- Tooltip -->
															<g class="chart-tooltip" opacity="0">
																<rect
																	x={point.x - 50}
																	y={point.y - 48}
																	width="100"
																	height="36"
																	rx="4"
																	fill="var(--bg-primary)"
																	stroke="var(--border-primary)"
																	stroke-width="1"
																/>
																<text
																	x={point.x}
																	y={point.y - 34}
																	text-anchor="middle"
																	fill="var(--text-primary)"
																	font-size="11"
																	font-weight="bold">{point.value}{graph.unit}</text
																>
																<text
																	x={point.x}
																	y={point.y - 20}
																	text-anchor="middle"
																	fill="var(--text-secondary)"
																	font-size="9">{point.date}</text
																>
															</g>
														</g>
													{/each}

													<!-- X-axis labels -->
													{#each chartPoints as point, i (point.x + '-' + point.y)}
														{#if i === 0 || i === chartPoints.length - 1 || chartPoints.length <= 10 || i % Math.ceil(chartPoints.length / 8) === 0}
															<text
																x={point.x}
																y={chartHeight + 20}
																text-anchor="middle"
																fill="var(--text-secondary)"
																font-size="9"
																transform="rotate(-30, {point.x}, {chartHeight + 20})"
																>{point.date}</text
															>
														{/if}
													{/each}

													<!-- Axes -->
													<line
														x1="0"
														y1={chartHeight}
														x2={chartWidth}
														y2={chartHeight}
														stroke="var(--border-primary)"
														stroke-width="1"
													/>
													<line
														x1="0"
														y1="0"
														x2="0"
														y2={chartHeight}
														stroke="var(--border-primary)"
														stroke-width="1"
													/>
												</svg>
											</div>
										</div>
									{/each}
								</div>
							{:else if includeTemperatureGraphs && filteredEntries.length > 0}
								<div
									class="mb-6 rounded-lg border border-dashed p-4 text-center"
									style="border-color: var(--border-primary);"
								>
									<p class="text-sm" style="color: var(--text-secondary);">
										No temperature graph data available. Graphs require at least 2 entries of the
										same log type with temperature fields.
									</p>
								</div>
							{/if}

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
								{#each Object.entries(groupedComponents) as [fieldType, componentGroup] (fieldType)}
									<div class="mb-6">
										<h3
											class="mb-3 border-b pb-2 text-lg font-bold"
											style="color: var(--text-primary); border-color: var(--border-primary);"
										>
											{fieldType} ({componentGroup.length} components)
										</h3>
										{#each componentGroup as component (component.componentId)}
											<div
												class="mb-4 rounded border p-4"
												style="border-color: var(--border-primary); background-color: var(--bg-secondary);"
											>
												<div class="mb-2 flex items-start justify-between">
													<div>
														<span class="font-medium" style="color: var(--text-primary);"
															>{formatTemplateName(
																component.entry.template_name,
																component.entry.period
															)}</span
														>
														<span class="ml-2 text-sm" style="color: var(--text-secondary);"
															>ID: {component.entry.id.slice(0, 8)}...</span
														>
													</div>
													<span
														class="rounded px-2 py-1 text-xs"
														style={component.entry.status === 'submitted'
															? 'background-color: var(--button-primary); color: var(--bg-primary);'
															: 'background-color: var(--orange); color: var(--bg-primary);'}
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
								{#each filteredEntries as entry (entry.id)}
									{@const excludedFieldTypes = getExcludedFieldTypes()}
									{@const shouldShowEntry = hasRemainingFields(
										entry.template_layout,
										excludedFieldTypes
									)}
									{#if shouldShowEntry}
										<div
											class="mb-4 rounded border p-4"
											style="border-color: var(--border-primary); background-color: var(--bg-secondary);"
										>
											<div class="mb-2 flex items-start justify-between">
												<div>
													<span class="font-medium" style="color: var(--text-primary);"
														>{formatTemplateName(entry.template_name, entry.period)}</span
													>
													<span class="ml-2 text-sm" style="color: var(--text-secondary);"
														>ID: {entry.id.slice(0, 8)}...</span
													>
												</div>
												<span
													class="rounded px-2 py-1 text-xs"
													style={entry.status === 'submitted'
														? 'background-color: var(--button-primary); color: var(--bg-primary);'
														: 'background-color: var(--orange); color: var(--bg-primary);'}
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
	.reports-page button:not(:disabled) {
		cursor: pointer;
	}

	/* Temperature graph chart styles */
	.chart-point:hover .chart-tooltip {
		opacity: 1 !important;
		transition: opacity 0.2s ease-in-out;
	}

	.chart-tooltip {
		pointer-events: none;
		transition: opacity 0.2s ease-in-out;
	}
</style>
