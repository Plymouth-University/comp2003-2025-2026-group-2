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
	type TemplateField = components['schemas']['TemplateField'];

	let { data } = $props<{ data: PageData }>();

	let entry = $derived<LogEntryResponse | null>(data.entry as LogEntryResponse | null);

	let templateLayout = $derived<TemplateField[]>(
		data.template?.template_layout || entry?.template_layout || []
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

	function isValidEntryData(data: unknown): data is Record<string, unknown> {
		return typeof data === 'object' && data !== null && !Array.isArray(data);
	}

	$effect(() => {
		if (entry && isValidEntryData(entry.entry_data) && Object.keys(entry.entry_data).length > 0) {
			entryData = { ...entry.entry_data } as Record<number, string | number | boolean>;
		} else if (templateLayout.length > 0) {
			const initialData: Record<number, string | number | boolean> = {};
			templateLayout.forEach((field: TemplateField, index: number) => {
				if (field.field_type === 'temperature') {
					const propValue = field.props?.value ? parseFloat(field.props.value) : undefined;
					initialData[index] = propValue ?? field.props?.min ?? 0;
				} else if (field.field_type === 'text' || field.field_type === 'text_input') {
					initialData[index] = field.props?.value || '';
				} else if (field.field_type === 'checkbox') {
					initialData[index] = field.props?.value === 'true';
				} else if (field.field_type === 'dropdown') {
					initialData[index] = field.props?.selected || (field.props?.options?.[0] ?? '');
				}
			});
			entryData = initialData;
		}
	});

	let mode = $derived(data.mode || 'view');
	let entryId = $derived(data.entryId || entry?.id);

	function validateForm(): string | null {
		for (let index = 0; index < templateLayout.length; index++) {
			const field = templateLayout[index];
			const value = entryData[index];
			const fieldLabel = field.props?.text || `Field ${index + 1}`;

			// Check required constraint based on field type
			if (field.props?.required === true) {
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
				if (field.props?.min_length != null) {
					const minLength = field.props.min_length;
					if (value.length < minLength) {
						return `${fieldLabel} must be at least ${minLength} characters long`;
					}
				}
				if (field.props?.max_length != null) {
					const maxLength = field.props.max_length;
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

			<div class="space-y-6">
				{#each templateLayout as field, index (index)}
					{#if field.field_type === 'temperature' && entryData[index] !== undefined}
						{@const tempMin =
							field.props?.min != null && !Number.isNaN(Number(field.props.min))
								? Number(field.props.min)
								: -20}
						{@const tempMax =
							field.props?.max != null && !Number.isNaN(Number(field.props.max))
								? Number(field.props.max)
								: 50}
						{@const fieldLabel = field.props?.text || `Field ${index + 1}`}
						{@const fieldUnit = field.props?.unit || '°C'}
						<TemperaturePicker
							bind:value={entryData[index] as number}
							min={tempMin}
							max={tempMax}
							label={fieldLabel}
							unit={fieldUnit}
							disabled={mode === 'view'}
						/>
					{:else if (field.field_type === 'text' || field.field_type === 'text_input') && entryData[index] !== undefined}
						{@const inputSize = field.props?.size != null ? Number(field.props.size) : 16}
						{@const safeSize = Number.isFinite(inputSize) && inputSize > 0 ? inputSize : 16}
						{@const fieldWeight = field.props?.weight || 'normal'}
						{@const fieldText = field.props?.text || `Field ${index + 1}`}
						{@const fieldFontFamily = field.props?.font_family || 'system-ui'}
						{@const fieldTextDecoration = field.props?.text_decoration || 'none'}
						{@const fieldColor = field.props?.color || ''}
						{@const fieldRequired = field.props?.required === true}
						{@const fieldMaxLength = field.props?.max_length ?? undefined}
						{@const fieldMinLength = field.props?.min_length ?? undefined}
						{@const fieldInputType = field.props?.input_type || 'text'}
						<UserTextInput
							bind:text={entryData[index] as string}
							size={safeSize}
							weight={fieldWeight}
							placeholder={fieldText}
							fontFamily={fieldFontFamily}
							textDecoration={fieldTextDecoration}
							color={fieldColor}
							required={fieldRequired}
							maxLength={fieldMaxLength}
							minLength={fieldMinLength}
							inputType={fieldInputType}
							disabled={mode === 'view'}
						/>
					{:else if field.field_type === 'checkbox' && entryData[index] !== undefined}
						{@const checkboxSize = field.props?.size != null ? String(field.props.size) : '16px'}
						{@const checkboxText = field.props?.text || `Field ${index + 1}`}
						{@const checkboxWeight = field.props?.weight || 'normal'}
						{@const checkboxColor = field.props?.color || ''}
						{@const checkboxRequired = field.props?.required === true}
						<UserCheckbox
							bind:checked={entryData[index] as boolean}
							text={checkboxText}
							size={checkboxSize}
							weight={checkboxWeight}
							color={checkboxColor}
							required={checkboxRequired}
							disabled={mode === 'view'}
						/>
					{:else if field.field_type === 'dropdown' && entryData[index] !== undefined}
						{@const dropdownOptions = field.props?.options || []}
						<UserDropdown
							bind:selected={entryData[index] as string}
							options={dropdownOptions}
							disabled={mode === 'view'}
						/>
					{:else if field.field_type === 'label'}
						{@const labelSize = field.props?.size != null ? Number(field.props.size) : 16}
						{@const safeLabelSize = Number.isFinite(labelSize) && labelSize > 0 ? labelSize : 16}
						{@const labelText = field.props?.text || `Field ${index + 1}`}
						{@const labelWeight = field.props?.weight || 'normal'}
						{@const labelFontFamily = field.props?.font_family || 'system-ui'}
						{@const labelTextDecoration = field.props?.text_decoration || 'none'}
						{@const labelColor = field.props?.color || ''}
						<UserTextLabel
							editable={false}
							text={labelText}
							size={safeLabelSize}
							weight={labelWeight}
							fontFamily={labelFontFamily}
							textDecoration={labelTextDecoration}
							color={labelColor}
						/>
					{/if}
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
