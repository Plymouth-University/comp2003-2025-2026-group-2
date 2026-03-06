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
		props: Record<string, unknown>;
	};

	// Type guard helpers for safely accessing props values
	function getPropAsString(props: Record<string, unknown>, key: string): string | undefined {
		const value = props[key];
		return typeof value === 'string' ? value : undefined;
	}

	function getPropAsNumber(props: Record<string, unknown>, key: string): number | undefined {
		const value = props[key];
		return typeof value === 'number' ? value : undefined;
	}

	function getPropAsStringOrBoolean(props: Record<string, unknown>, key: string): string | boolean | undefined {
		const value = props[key];
		return typeof value === 'string' || typeof value === 'boolean' ? value : undefined;
	}

	function getPropAsArray<T = unknown>(props: Record<string, unknown>, key: string): T[] | undefined {
		const value = props[key];
		return Array.isArray(value) ? value : undefined;
	}

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
				const props = field.props || {};
				if (field.field_type === 'temperature') {
					const propValue = getPropAsStringOrBoolean(props, 'value');
					const numValue = typeof propValue === 'string' ? parseFloat(propValue) : undefined;
					initialData[index] = numValue ?? (typeof propValue === 'number' ? propValue : 0);
				} else if (field.field_type === 'text' || field.field_type === 'text_input') {
					const propValue = getPropAsString(props, 'value');
					initialData[index] = propValue || '';
				} else if (field.field_type === 'checkbox') {
					const propValue = getPropAsStringOrBoolean(props, 'value');
					initialData[index] = propValue === 'true' || propValue === true;
				} else if (field.field_type === 'dropdown') {
					const propSelected = getPropAsString(props, 'selected');
					const propOptions = getPropAsArray(props, 'options');
					const firstOption = propOptions?.[0];
					const firstOptionValue = typeof firstOption === 'object' && firstOption !== null && 'value' in firstOption 
						? (firstOption as { value?: unknown }).value 
						: firstOption;
					initialData[index] = propSelected || (typeof firstOptionValue === 'string' ? firstOptionValue : '');
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
			const fieldLabel = getPropAsString(field.props, 'text') || `Field ${index + 1}`;

			// Check required constraint based on field type
			const requiredProp = getPropAsStringOrBoolean(field.props, 'required');
			if (requiredProp === true || requiredProp === 'true') {
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
				const minLengthProp = getPropAsNumber(field.props, 'min_length') ?? 
					(typeof field.props['min_length'] === 'string' ? Number(field.props['min_length']) : undefined);
				if (minLengthProp != null) {
					const minLength = minLengthProp;
					if (value.length < minLength) {
						return `${fieldLabel} must be at least ${minLength} characters long`;
					}
				}
				const maxLengthProp = getPropAsNumber(field.props, 'max_length') ?? 
					(typeof field.props['max_length'] === 'string' ? Number(field.props['max_length']) : undefined);
				if (maxLengthProp != null) {
					const maxLength = maxLengthProp;
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
				{#each templateLayout as field, index (field.field_id || index)}
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
							label={field.props.text || `Field ${index + 1}`}
							unit={field.props.unit || '°C'}
							disabled={mode === 'view'}
						/>
					{:else if (field.field_type === 'text' || field.field_type === 'text_input') && entryData[index] !== undefined}
						{@const inputSize = field.props.size != null ? Number(field.props.size) : 16}
						{@const safeSize = Number.isFinite(inputSize) && inputSize > 0 ? inputSize : 16}
						<UserTextInput
							bind:text={entryData[index] as string}
							size={safeSize}
							weight={field.props.weight ?? 'normal'}
							placeholder={field.props.text || `Field ${index + 1}`}
							fontFamily={field.props.font_family ?? 'system-ui'}
							textDecoration={field.props.text_decoration ?? 'none'}
							color={field.props.color ?? ''}
							required={field.props.required === true || field.props.required === 'true'}
							maxLength={field.props.max_length != null
								? Number(field.props.max_length)
								: undefined}
							minLength={field.props.min_length != null
								? Number(field.props.min_length)
								: undefined}
							inputType={field.props.input_type ?? 'text'}
							disabled={mode === 'view'}
						/>
					{:else if field.field_type === 'checkbox' && entryData[index] !== undefined}
						{@const checkboxSize = field.props.size != null ? String(field.props.size) : '16px'}
						<UserCheckbox
							bind:checked={entryData[index] as boolean}
							text={field.props.text || `Field ${index + 1}`}
							size={checkboxSize}
							weight={field.props.weight ?? 'normal'}
							color={field.props.color ?? ''}
							required={field.props.required === true || field.props.required === 'true'}
							disabled={mode === 'view'}
						/>
					{:else if field.field_type === 'dropdown' && entryData[index] !== undefined}
						<UserDropdown
							bind:selected={entryData[index] as string}
							options={field.props.options || []}
							disabled={mode === 'view'}
						/>
					{:else if field.field_type === 'label'}
						{@const labelSize = field.props.size != null ? Number(field.props.size) : 16}
						{@const safeLabelSize = Number.isFinite(labelSize) && labelSize > 0 ? labelSize : 16}
						<UserTextLabel
							editable={false}
							text={field.props.text || `Field ${index + 1}`}
							size={safeLabelSize}
							weight={field.props.weight ?? 'normal'}
							fontFamily={field.props.font_family ?? 'system-ui'}
							textDecoration={field.props.text_decoration ?? 'none'}
							color={field.props.color ?? ''}
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
