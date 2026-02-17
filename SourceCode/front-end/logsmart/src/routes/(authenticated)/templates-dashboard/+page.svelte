<script lang="ts">
	import { goto } from '$app/navigation';
	import { api } from '$lib/api';
	import type { components } from '$lib/api-types';
	import type { Template, TemplateSchedule, DayOfWeek } from './types';
	import TemplateRow from './TemplateRow.svelte';
	import TemplateSettingsWizard from './TemplateSettingsWizard.svelte';
	import type { PageData } from './$types';

	type ApiTemplateInfo = components['schemas']['TemplateInfo'];
	type ApiSchedule = components['schemas']['Schedule'];

	let { data }: { data: PageData } = $props();
	let user = $derived(data?.user);
	let isReadonlyHQ = $derived(user?.role === 'staff' && !user?.branch_id);

	const dayNumberToName: DayOfWeek[] = [
		'sunday',
		'monday',
		'tuesday',
		'wednesday',
		'thursday',
		'friday',
		'saturday'
	];

	function mapApiScheduleToLocal(apiSchedule: ApiSchedule): TemplateSchedule {
		const frequency = apiSchedule.frequency.toLowerCase() as TemplateSchedule['frequency'];
		const schedule: TemplateSchedule = { frequency };

		if (apiSchedule.days_of_week && apiSchedule.days_of_week.length > 0) {
			schedule.daysOfWeek = apiSchedule.days_of_week.map((d) => dayNumberToName[d]);
		}
		if (apiSchedule.day_of_week !== null && apiSchedule.day_of_week !== undefined) {
			schedule.dayOfWeek = dayNumberToName[apiSchedule.day_of_week];
		}
		if (apiSchedule.day_of_month !== null && apiSchedule.day_of_month !== undefined) {
			schedule.dayOfMonth = apiSchedule.day_of_month;
		}
		if (apiSchedule.month_of_year !== null && apiSchedule.month_of_year !== undefined) {
			schedule.monthOfYear = apiSchedule.month_of_year;
		}

		return schedule;
	}

	const dayNameToNumber: Record<DayOfWeek, number> = {
		sunday: 0,
		monday: 1,
		tuesday: 2,
		wednesday: 3,
		thursday: 4,
		friday: 5,
		saturday: 6
	};

	function mapLocalScheduleToApi(schedule: TemplateSchedule): ApiSchedule {
		const apiSchedule: ApiSchedule = {
			frequency: (schedule.frequency.charAt(0).toUpperCase() +
				schedule.frequency.slice(1)) as ApiSchedule['frequency']
		};

		if (schedule.daysOfWeek && schedule.daysOfWeek.length > 0) {
			apiSchedule.days_of_week = schedule.daysOfWeek.map((d) => dayNameToNumber[d]);
		}
		if (schedule.dayOfWeek) {
			apiSchedule.day_of_week = dayNameToNumber[schedule.dayOfWeek];
		}
		if (schedule.dayOfMonth !== undefined) {
			apiSchedule.day_of_month = schedule.dayOfMonth;
		}
		if (schedule.monthOfYear !== undefined) {
			apiSchedule.month_of_year = schedule.monthOfYear;
		}

		return apiSchedule;
	}

	function mapApiTemplateToLocal(apiTemplate: ApiTemplateInfo): Template {
		return {
			id: apiTemplate.template_name,
			name: apiTemplate.template_name,
			createdAt: apiTemplate.created_at,
			updatedAt: apiTemplate.updated_at,
			schedule: mapApiScheduleToLocal(apiTemplate.schedule),
			layout: []
		};
	}

	let templates = $state<Template[]>([]);
	let loading = $state(true);
	let error = $state<string | null>(null);

	async function fetchTemplates() {
		loading = true;
		error = null;
		const { data, error: apiError } = await api.GET('/logs/templates/all');
		if (apiError) {
			error = 'Failed to load templates';
			loading = false;
			return;
		}
		if (data?.templates) {
			templates = data.templates.map(mapApiTemplateToLocal);
		}
		loading = false;
	}

	$effect(() => {
		fetchTemplates();
	});

	let selectedTemplate = $state<Template | null>(null);
	let settingsWizardOpen = $state(false);
	let deleteConfirmOpen = $state(false);
	let templateToDelete = $state<Template | null>(null);
	let searchQuery = $state('');

	const filteredTemplates = $derived(
		templates.filter((t) => t.name.toLowerCase().includes(searchQuery.toLowerCase()))
	);

	function handleEdit(template: Template) {
		goto(`/template-designer?id=${template.id}`);
	}

	function handleSettings(template: Template) {
		selectedTemplate = template;
		settingsWizardOpen = true;
	}

	async function handleSaveSchedule(schedule: TemplateSchedule) {
		if (!selectedTemplate) return;

		const apiSchedule = mapLocalScheduleToApi(schedule);
		const { error: apiError } = await api.PUT('/logs/templates/update', {
			body: {
				template_name: selectedTemplate.name,
				schedule: apiSchedule
			}
		});

		if (apiError) {
			console.error('Failed to update schedule:', apiError);
			return;
		}

		const idx = templates.findIndex((t) => t.id === selectedTemplate!.id);
		if (idx !== -1) {
			templates[idx] = {
				...templates[idx],
				schedule,
				updatedAt: new Date().toISOString()
			};
		}
		selectedTemplate = null;
	}

	function handleDelete(template: Template) {
		templateToDelete = template;
		deleteConfirmOpen = true;
	}

	async function confirmDelete() {
		if (!templateToDelete) return;

		const { error: apiError } = await api.DELETE('/logs/templates', {
			params: {
				query: {
					template_name: templateToDelete.name
				}
			}
		});

		if (apiError) {
			console.error('Failed to delete template:', apiError);
			error = 'Failed to delete template';
			templateToDelete = null;
			deleteConfirmOpen = false;
			return;
		}

		templates = templates.filter((t) => t.id !== templateToDelete!.id);
		templateToDelete = null;
		deleteConfirmOpen = false;
	}

	function cancelDelete() {
		templateToDelete = null;
		deleteConfirmOpen = false;
	}

	function handleCreateNew() {
		goto('/template-designer');
	}
</script>

<svelte:head>
	<title>Templates Dashboard</title>
</svelte:head>

<div class="min-h-screen" style="background-color: var(--bg-secondary);">
	<div class="mx-auto max-w-5xl px-6 py-8">
		<div class="mb-8 flex items-center justify-between">
			<h1 class="text-3xl font-bold" style="color: var(--text-primary);">Templates Dashboard</h1>
			{#if !isReadonlyHQ}
				<button
					type="button"
					class="btn-create rounded px-6 py-3 text-white md:font-medium"
					onclick={handleCreateNew}
				>
					<span class="hidden sm:inline">➕ Create New Template</span>
					<span class="sm:hidden">➕</span>
				</button>
			{/if}
		</div>

		<div class="mb-6">
			<input
				type="text"
				bind:value={searchQuery}
				placeholder="Search templates..."
				class="w-full rounded-lg border-2 px-4 py-3"
				style="background-color: var(--bg-primary); border-color: var(--border-primary); color: var(--text-primary);"
			/>
		</div>

		{#if loading}
			<div
				class="rounded-lg border-2 px-6 py-12 text-center"
				style="background-color: var(--bg-primary); border-color: var(--border-primary);"
			>
				<p class="text-lg" style="color: var(--text-secondary);">Loading templates...</p>
			</div>
		{:else if error}
			<div
				class="rounded-lg border-2 px-6 py-12 text-center"
				style="background-color: var(--bg-primary); border-color: #d9534f;"
			>
				<p class="text-lg" style="color: #d9534f;">{error}</p>
				<button
					type="button"
					class="btn-retry mt-4 rounded px-4 py-2 font-medium text-white"
					onclick={fetchTemplates}
				>
					Retry
				</button>
			</div>
		{:else if filteredTemplates.length === 0}
			<div
				class="rounded-lg border-2 px-6 py-12 text-center"
				style="background-color: var(--bg-primary); border-color: var(--border-primary);"
			>
				{#if searchQuery}
					<p class="text-lg" style="color: var(--text-secondary);">
						No templates found matching "{searchQuery}"
					</p>
				{:else}
					<p class="text-lg" style="color: var(--text-secondary);">
						No templates yet. Create your first template to get started!
					</p>
				{/if}
			</div>
		{:else}
			<div class="space-y-4">
			{#each filteredTemplates as template (template.id)}
				<TemplateRow
					{template}
					onEdit={handleEdit}
					onSettings={handleSettings}
					onDelete={handleDelete}
					{isReadonlyHQ}
				/>
			{/each}
			</div>
		{/if}

		<div class="mt-6 text-sm" style="color: var(--text-secondary);">
			{filteredTemplates.length} template{filteredTemplates.length !== 1 ? 's' : ''}
		</div>
	</div>
</div>

{#if selectedTemplate}
	<TemplateSettingsWizard
		template={selectedTemplate}
		bind:isOpen={settingsWizardOpen}
		onSave={handleSaveSchedule}
	/>
{/if}

{#if deleteConfirmOpen && templateToDelete}
	<div
		class="fixed inset-0 z-50 flex items-center justify-center bg-black/50"
		onclick={(e) => e.target === e.currentTarget && cancelDelete()}
		onkeydown={(e) => e.key === 'Escape' && cancelDelete()}
		role="dialog"
		aria-modal="true"
		tabindex="-1"
	>
		<div
			class="w-full max-w-md rounded-lg border-2 shadow-xl"
			style="background-color: var(--bg-primary); border-color: var(--border-primary);"
		>
			<div class="px-6 py-6">
				<h2 class="mb-4 text-xl font-bold" style="color: var(--text-primary);">Delete Template?</h2>
				<p style="color: var(--text-secondary);">
					Are you sure you want to delete "{templateToDelete.name}"? This action cannot be undone.
				</p>
			</div>
			<div
				class="flex justify-end gap-3 border-t-2 px-6 py-4"
				style="border-color: var(--border-primary);"
			>
				<button
					type="button"
					class="btn-cancel rounded px-4 py-2 font-medium"
					onclick={cancelDelete}
				>
					Cancel
				</button>
				<button
					type="button"
					class="btn-confirm-delete rounded px-4 py-2 font-medium text-white"
					onclick={confirmDelete}
				>
					Delete
				</button>
			</div>
		</div>
	</div>
{/if}

<style>
	.btn-create {
		background-color: #5cb85c;
		transition: background-color 0.15s ease;
	}

	.btn-create:hover {
		background-color: #449d44;
	}

	.btn-create:active {
		background-color: #398439;
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

	.btn-confirm-delete {
		background-color: #d9534f;
		transition: background-color 0.15s ease;
	}

	.btn-confirm-delete:hover {
		background-color: #c9302c;
	}

	.btn-confirm-delete:active {
		background-color: #ac2925;
	}

	.btn-retry {
		background-color: #337ab7;
		transition: background-color 0.15s ease;
	}

	.btn-retry:hover {
		background-color: #286090;
	}

	.btn-retry:active {
		background-color: #204d74;
	}
</style>
