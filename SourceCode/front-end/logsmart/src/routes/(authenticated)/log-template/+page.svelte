<script lang="ts">
	import type { PageData } from './$types';
	import { goto, invalidateAll } from '$app/navigation';
	import TemperaturePicker from '$lib/components/temperature_picker.svelte';
	import UserCheckbox from '$lib/components/user_checkbox.svelte';
	import UserDropdown from '$lib/components/user_dropdown.svelte';
	import UserTextInput from '$lib/components/user_text_input.svelte';
	import UserTextLabel from '$lib/components/user_text_label.svelte';

	let { data } = $props<{ data: PageData }>();

	let templateLayout = $derived(
		data.template?.template_layout || data.entry?.template_layout || []
	);
	let rawTemplateName = $derived(
		data.template?.template_name || data.entry?.template_name || 'Log Template'
	);
	let period = $derived(data.entry?.period);
	let templateName = $derived(
		period && rawTemplateName.includes('{period}')
			? rawTemplateName.replace('{period}', period)
			: rawTemplateName
	);

	let entryData = $state<Record<number, string | number | boolean>>({});

	$effect(() => {
		if (data.entry?.entry_data) {
			entryData = { ...data.entry.entry_data };
		} else if (templateLayout.length > 0) {
			const initialData: Record<number, string | number | boolean> = {};
			templateLayout.forEach((field: any, index: number) => {
				if (field.field_type === 'temperature') {
					initialData[index] = field.props.value !== undefined ? parseFloat(field.props.value) : 0;
				} else if (field.field_type === 'text' || field.field_type === 'text_input') {
					initialData[index] = field.props.value || '';
				} else if (field.field_type === 'checkbox') {
					initialData[index] = field.props.value === 'true' || field.props.value === true;
				} else if (field.field_type === 'dropdown') {
					initialData[index] = field.props.selected || (field.props.options?.[0] ?? '');
				}
			});
			entryData = initialData;
		}
	});

	let mode = $derived(data.mode || 'view');
	let entryId = $derived(data.entryId || data.entry?.id);

	function validateForm(): string | null {
		for (let index = 0; index < templateLayout.length; index++) {
			const field = templateLayout[index];
			const value = entryData[index];
			const fieldLabel = field.props.text || `Field ${index + 1}`;

			// Check required constraint
			if (field.props.required === true || field.props.required === 'true') {
				if (value === undefined || value === null || value === '') {
					return `${fieldLabel} is required`;
				}
			}

			// Check length constraints for text inputs
			if (
				(field.field_type === 'text' || field.field_type === 'text_input') &&
				typeof value === 'string'
			) {
				if (field.props.min_length) {
					const minLength = Number(field.props.min_length);
					if (value.length < minLength) {
						return `${fieldLabel} must be at least ${minLength} characters long`;
					}
				}
				if (field.props.max_length) {
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
				await invalidateAll();
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
			{#if mode === 'view' && data.entry?.status === 'submitted'}
				<div
					class="mb-4 rounded p-4"
					style="background-color: #e8f5e9; border: 1px solid #4caf50; color: #2e7d32;"
				>
					This log has been submitted and cannot be edited.
				</div>
			{:else if mode === 'edit' && data.entry?.status === 'submitted'}
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
						<TemperaturePicker
							bind:value={entryData[index] as number}
							min={Number(field.props.min) ?? -20}
							max={Number(field.props.max) ?? 50}
							label={field.props.text || `Field ${index + 1}`}
							unit={field.props.unit || '°C'}
							disabled={mode === 'view'}
						/>
					{:else if (field.field_type === 'text' || field.field_type === 'text_input') && entryData[index] !== undefined}
						<UserTextInput
							bind:text={entryData[index] as string}
							size={Number(field.props.size) ?? 16}
							weight={String(field.props.weight) || 'normal'}
							placeholder={field.props.text || `Field ${index + 1}`}
							fontFamily={String(field.props.font_family) || 'system-ui'}
							textDecoration={String(field.props.text_decoration) || 'none'}
							color={String(field.props.color) || ''}
							required={field.props.required === true || field.props.required === 'true'}
							maxLength={field.props.max_length ? Number(field.props.max_length) : undefined}
							minLength={field.props.min_length ? Number(field.props.min_length) : undefined}
							inputType={String(field.props.input_type) || 'text'}
							disabled={mode === 'view'}
						/>
					{:else if field.field_type === 'checkbox' && entryData[index] !== undefined}
						<UserCheckbox
							bind:checked={entryData[index] as boolean}
							text={field.props.text || `Field ${index + 1}`}
							size={String(field.props.size) || '16px'}
							weight={String(field.props.weight) || 'normal'}
							color={String(field.props.color) || ''}
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
						<UserTextLabel
							editable={false}
							text={field.props.text || `Field ${index + 1}`}
							size={Number(field.props.size) ?? 16}
							weight={String(field.props.weight) || 'normal'}
							fontFamily={String(field.props.font_family) || 'system-ui'}
							textDecoration={String(field.props.text_decoration) || 'none'}
							color={String(field.props.color) || ''}
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
