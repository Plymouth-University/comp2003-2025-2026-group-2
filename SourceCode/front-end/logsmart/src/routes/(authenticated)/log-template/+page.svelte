<script lang="ts">
	import type { PageData } from './$types';
	import type { components } from '$lib/api-types';
	import { goto } from '$app/navigation';
	import TemperaturePicker from '$lib/components/temperature_picker.svelte';
	import UserCheckbox from '$lib/components/user_checkbox.svelte';
	import UserDropdown from '$lib/components/user_dropdown.svelte';
	import UserTextInput from '$lib/components/user_text_input.svelte';
	import UserTextLabel from '$lib/components/user_text_label.svelte';

	type LogEntryResponse = components['schemas']['LogEntryResponse'];

	let { data } = $props<{ data: PageData }>();

	let entry = $derived<LogEntryResponse | null>(data.entry as LogEntryResponse | null);

	type TemplateField = {
		field_id?: string;
		field_type: string;
		position?: { x: number; y: number };
		props: Record<string, unknown>;
	};

	let templateLayout = $derived<TemplateField[]>(
		entry?.template_layout || data.template?.template_layout || []
	);
	let rawTemplateName = $derived(
		data.template?.template_name || entry?.template_name || 'Log Template'
	);
	let period = $derived(entry?.period);
	let templateName = $derived(
		period && rawTemplateName.includes('{period}')
			? rawTemplateName.replace('{period}', period)
			: rawTemplateName
	);

	let entryData = $state<Record<number, string | number | boolean>>({});

	// Calculate canvas height based on field positions
	let canvasHeight = $derived(() => {
		if (!templateLayout || templateLayout.length === 0) return 500;
		const maxY = Math.max(...templateLayout.map((f: any) => (f.position?.y || 0) + 100));
		return Math.max(500, maxY + 100); // Add padding at bottom
	});

	function isValidEntryData(data: unknown): data is Record<string, unknown> {
		return typeof data === 'object' && data !== null && !Array.isArray(data);
	}

	function getStringProp(props: Record<string, unknown>, key: string, fallback = ''): string {
		const value = props[key];
		return typeof value === 'string' ? value : fallback;
	}

	function getStringArrayProp(props: Record<string, unknown>, key: string): string[] {
		const value = props[key];
		if (!Array.isArray(value)) return [];
		return value.filter((item): item is string => typeof item === 'string');
	}

	function buildInitialData(layout: TemplateField[]): Record<number, string | number | boolean> {
		const initialData: Record<number, string | number | boolean> = {};
		layout.forEach((field: TemplateField, index: number) => {
			const props = field.props || {};
			if (field.field_type === 'temperature') {
				const val = props.value;
				initialData[index] = val !== undefined && typeof val === 'string' ? parseFloat(val) : 0;
			} else if (field.field_type === 'text' || field.field_type === 'text_input') {
				initialData[index] = typeof props.value === 'string' ? props.value : '';
			} else if (field.field_type === 'checkbox') {
				initialData[index] = props.value === 'true' || props.value === true;
			} else if (field.field_type === 'dropdown') {
				const sel = props.selected;
				const opts = Array.isArray(props.options) ? props.options : [];
				initialData[index] = typeof sel === 'string' ? sel : ((opts[0] as string) ?? '');
			}
		});
		return initialData;
	}

	function toEntryDataValue(value: unknown): string | number | boolean {
		if (typeof value === 'string' || typeof value === 'number' || typeof value === 'boolean') {
			return value;
		}
		return '';
	}

	$effect(() => {
		if (templateLayout.length === 0) {
			entryData = {};
			return;
		}

		const initialData = buildInitialData(templateLayout);

		if (entry && isValidEntryData(entry.entry_data) && Object.keys(entry.entry_data).length > 0) {
			const mergedData = { ...initialData };
			for (const [key, rawValue] of Object.entries(entry.entry_data)) {
				const numericKey = Number(key);
				if (Number.isInteger(numericKey) && numericKey >= 0) {
					mergedData[numericKey] = toEntryDataValue(rawValue);
				}
			}
			entryData = mergedData;
		} else {
			entryData = initialData;
		}
	});

	let mode = $derived(data.mode || 'view');
	let entryId = $derived(data.entryId || entry?.id);

	function validateForm(): string | null {
		for (let index = 0; index < templateLayout.length; index++) {
			const field = templateLayout[index];
			const value = entryData[index];
			const fieldLabel = getStringProp(field.props, 'text', `Field ${index + 1}`);

			// Check required constraint based on field type
			if (field.props.required === true || field.props.required === 'true') {
				let isEmpty = false;

				if (field.field_type === 'checkbox') {
					// Checkbox must be explicitly true
					isEmpty = value !== true;
				} else if (field.field_type === 'temperature') {
					// Temperature must be a finite number
					isEmpty = typeof value !== 'number' || !Number.isFinite(value);
				} else if (field.field_type === 'dropdown') {
					// Dropdown must not be null or empty string
					isEmpty = value == null || value === '';
				} else if (field.field_type === 'text' || field.field_type === 'text_input') {
					// Text must be a non-empty string (after trimming)
					isEmpty = typeof value !== 'string' || value.trim() === '';
				}

				if (isEmpty) {
					return `${fieldLabel} is required`;
				}
			}

			// Check length constraints for text inputs
			if (
				(field.field_type === 'text' || field.field_type === 'text_input') &&
				typeof value === 'string'
			) {
				if (field.props.min_length != null) {
					const minLength = Number(field.props.min_length);
					if (value.length < minLength) {
						return `${fieldLabel} must be at least ${minLength} characters long`;
					}
				}
				if (field.props.max_length != null) {
					const maxLength = Number(field.props.max_length);
					if (value.length > maxLength) {
						return `${fieldLabel} must not exceed ${maxLength} characters`;
					}
				}
			}
		}
		return null;
	}

	async function handleSave() {
		if (!entryId) return;

		try {
			const response = await fetch(`/api/logs/entries/${entryId}`, {
				method: 'PUT',
				headers: {
					'Content-Type': 'application/json'
				},
				body: JSON.stringify({
					entry_data: entryData
				})
			});

			if (response.ok) {
				alert('Log saved successfully');
			} else {
				alert('Failed to save log');
			}
		} catch (error) {
			console.error('Error saving log:', error);
			alert('Error saving log');
		}
	}

	async function handleSubmit() {
		if (!entryId) return;

		// Validate form before submission
		const validationError = validateForm();
		if (validationError) {
			alert(validationError);
			return;
		}

		await handleSave();

		try {
			const submitUrl = `/api/logs/entries/${entryId}/submit`;

			const response = await fetch(submitUrl, {
				method: 'POST',
				headers: {
					'Content-Type': 'application/json'
				}
			});

			if (response.ok) {
				await response.json();

				alert('Log submitted successfully');
				goto('/logs-list', { replaceState: false, invalidateAll: true });
			} else {
				const error = await response.text();
				console.error('Submit failed:', error);
				alert('Failed to submit log');
			}
		} catch (error) {
			console.error('Error submitting log:', error);
			alert('Error submitting log');
		}
	}
</script>

<svelte:head>
	<title>{templateName}</title>
</svelte:head>
<main>
	{#if data.error}
		<div
			class="mb-4 rounded p-4"
			style="background-color: #fee; border: 1px solid #fcc; color: #c00; margin: 2rem;"
		>
			{data.error}
		</div>
	{:else}
		<h1
			class="mb-6 text-center text-3xl font-bold"
			style="padding-top:1%; color: var(--text-primary)"
		>
			{templateName}
		</h1>
		<div
			class="rounded-lg border-2 p-8"
			style="border-color: var(--border-color); margin-left:10%; margin-right:10%; background-color: var(--bg-primary);"
		>
			{#if mode === 'view' && entry?.status === 'submitted'}
				<div
					class="mb-4 rounded p-4"
					style="background-color: #e8f5e9; border: 1px solid #4caf50; color: #2e7d32;"
				>
					This log has been submitted and cannot be edited.
				</div>
			{:else if mode === 'edit' && entry?.status === 'submitted'}
				<div
					class="mb-4 rounded p-4"
					style="background-color: #fff3cd; border: 1px solid #ffc107; color: #856404;"
				>
					Warning: This log was previously submitted and has been reopened for editing.
				</div>
			{/if}

			<!-- Canvas with positioned fields -->
			<div
				class="relative"
				style="min-height: {canvasHeight()}px; border: 1px dashed var(--border-color); border-radius: 4px; padding: 20px;"
			>
				{#each templateLayout as field, index (field.field_id || index)}
					{@const x = field.position?.x ?? 0}
					{@const y = field.position?.y ?? 0}
					<div class="absolute" style="left: {x}px; top: {y}px;">
						{#if field.field_type === 'temperature' && entryData[index] !== undefined}
							{@const tempMin =
								field.props.min != null && !Number.isNaN(Number(field.props.min))
									? Number(field.props.min)
									: -20}
							{@const tempMax =
								field.props.max != null && !Number.isNaN(Number(field.props.max))
									? Number(field.props.max)
									: 50}
							<TemperaturePicker
								bind:value={entryData[index] as number}
								min={tempMin}
								max={tempMax}
								label={getStringProp(field.props, 'text', `Field ${index + 1}`)}
								unit={getStringProp(field.props, 'unit', '°C')}
								disabled={mode === 'view'}
							/>
						{:else if (field.field_type === 'text' || field.field_type === 'text_input') && entryData[index] !== undefined}
							{@const inputSize = field.props.size != null ? Number(field.props.size) : 16}
							{@const safeSize = Number.isFinite(inputSize) && inputSize > 0 ? inputSize : 16}
							<UserTextInput
								bind:text={entryData[index] as string}
								size={safeSize}
								weight={getStringProp(field.props, 'weight', 'normal')}
								placeholder={getStringProp(field.props, 'text', `Field ${index + 1}`)}
								fontFamily={getStringProp(field.props, 'font_family', 'system-ui')}
								textDecoration={getStringProp(field.props, 'text_decoration', 'none')}
								color={getStringProp(field.props, 'color')}
								required={field.props.required === true || field.props.required === 'true'}
								maxLength={field.props.max_length != null
									? Number(field.props.max_length)
									: undefined}
								minLength={field.props.min_length != null
									? Number(field.props.min_length)
									: undefined}
								inputType={getStringProp(field.props, 'input_type', 'text')}
								disabled={mode === 'view'}
							/>
						{:else if field.field_type === 'checkbox' && entryData[index] !== undefined}
							{@const checkboxSize = field.props.size != null ? String(field.props.size) : '16px'}
							<UserCheckbox
								bind:checked={entryData[index] as boolean}
								text={getStringProp(field.props, 'text', `Field ${index + 1}`)}
								size={checkboxSize}
								weight={getStringProp(field.props, 'weight', 'normal')}
								color={getStringProp(field.props, 'color')}
								required={field.props.required === true || field.props.required === 'true'}
								disabled={mode === 'view'}
							/>
						{:else if field.field_type === 'dropdown' && entryData[index] !== undefined}
							<UserDropdown
								bind:selected={entryData[index] as string}
								options={getStringArrayProp(field.props, 'options')}
								disabled={mode === 'view'}
							/>
						{:else if field.field_type === 'label'}
							{@const labelSize = field.props.size != null ? Number(field.props.size) : 16}
							{@const safeLabelSize = Number.isFinite(labelSize) && labelSize > 0 ? labelSize : 16}
							<UserTextLabel
								editable={false}
								text={getStringProp(field.props, 'text', `Field ${index + 1}`)}
								size={safeLabelSize}
								weight={getStringProp(field.props, 'weight', 'normal')}
								fontFamily={getStringProp(field.props, 'font_family', 'system-ui')}
								textDecoration={getStringProp(field.props, 'text_decoration', 'none')}
								color={getStringProp(field.props, 'color')}
							/>
						{/if}
					</div>
				{/each}
			</div>

			{#if mode !== 'view'}
				<div class="mt-8 flex justify-end gap-4">
					<button
						onclick={handleSave}
						class="rounded px-6 py-2 font-semibold hover:opacity-80"
						style="background-color: #3D7A82; color: white;"
					>
						Save Draft
					</button>
					<button
						onclick={handleSubmit}
						class="rounded px-6 py-2 font-semibold hover:opacity-80"
						style="background-color: #4caf50; color: white;"
					>
						Submit Log
					</button>
				</div>
			{/if}
		</div>

		<div class="mt-4 flex justify-center gap-8">
			<a href="/logs-list" class="rounded px-6 py-2" style="background-color: #ddd; color: #000;">
				Back to Logs
			</a>
		</div>
	{/if}
</main>
