<script lang="ts">
	import type { Template } from './types';
	import { DEFAULT_TEMPLATE_BLUEPRINTS } from './defaultTemplates';

	let selectedDefaultTemplateId = $state<string>(DEFAULT_TEMPLATE_BLUEPRINTS[0]?.id ?? '');
	let selectedDefaultTemplate = $derived(
		DEFAULT_TEMPLATE_BLUEPRINTS.find((t) => t.id === selectedDefaultTemplateId)
	);

	const checklistTemplates = DEFAULT_TEMPLATE_BLUEPRINTS.filter(
		(template) => template.category === 'checklist'
	);
	const temperatureTemplates = DEFAULT_TEMPLATE_BLUEPRINTS.filter(
		(template) => template.category === 'temperature'
	);
	const combinedTemplates = DEFAULT_TEMPLATE_BLUEPRINTS.filter(
		(template) => template.category === 'combined'
	);

	let {
		templates,
		onCreateNew,
		onUseDefaultTemplate,
		onSelectTemplate,
		currentTemplateName = '',
		isNewTemplate = false
	}: {
		templates: Template[];
		onCreateNew: () => void;
		onUseDefaultTemplate: (templateId: string) => void;
		onSelectTemplate: (templateName: string) => void;
		currentTemplateName?: string;
		isNewTemplate?: boolean;
	} = $props();

	function handleUseSelectedDefaultTemplate() {
		if (!selectedDefaultTemplateId) return;
		onUseDefaultTemplate(selectedDefaultTemplateId);
	}
</script>

<div
	class="flex flex-col"
	style="border-color: var(--border-primary); background-color: var(--bg-primary);"
>
	<div class="p-6">
		<h2 class="mb-4 text-2xl font-bold" style="color: var(--text-primary);">Templates</h2>

		<button
			class="btn-create mb-6 w-full cursor-pointer rounded px-4 py-2 font-medium text-white"
			onclick={onCreateNew}
		>
			+ Create New
		</button>

		<div class="mb-6 border-2 px-4 py-4" style="border-color: var(--border-primary);">
			<h3 class="mb-2 text-sm font-semibold uppercase" style="color: var(--text-secondary);">
				Start From Default
			</h3>

			<select
				class="mb-3 h-9 w-full max-w-60 rounded border-2 px-2 py-1 text-xs"
				style="border-color: var(--border-primary); background-color: var(--bg-primary); color: var(--text-primary);"
				bind:value={selectedDefaultTemplateId}
			>
				{#if checklistTemplates.length > 0}
					<optgroup label="Checklists">
						{#each checklistTemplates as template (template.id)}
							<option value={template.id}>{template.name}</option>
						{/each}
					</optgroup>
				{/if}
				{#if temperatureTemplates.length > 0}
					<optgroup label="Temperature Logs">
						{#each temperatureTemplates as template (template.id)}
							<option value={template.id}>{template.name}</option>
						{/each}
					</optgroup>
				{/if}
				{#if combinedTemplates.length > 0}
					<optgroup label="Combined">
						{#each combinedTemplates as template (template.id)}
							<option value={template.id}>{template.name}</option>
						{/each}
					</optgroup>
				{/if}
			</select>

			{#if selectedDefaultTemplate}
				<p class="mb-3 text-xs" style="color: var(--text-secondary);">
					{selectedDefaultTemplate.description}
				</p>
			{/if}

			<button
				type="button"
				class="btn-default w-full rounded px-4 py-2 text-sm font-semibold text-white"
				onclick={handleUseSelectedDefaultTemplate}
				disabled={!selectedDefaultTemplateId}
			>
				Use Template
			</button>
		</div>

		<div class="border-2" style="border-color: var(--border-primary);">
			<ul class="divide-y" style="border-color: var(--border-secondary);">
				{#if isNewTemplate}
					<li
						class="flex min-w-0 items-center gap-3 px-4 py-3"
						style="background-color: var(--bg-secondary);"
					>
						<div
							class="flex h-5 w-5 items-center justify-center border-2"
							style="border-color: var(--border-primary);"
						>
							<svg
								width="12"
								height="12"
								viewBox="0 0 12 12"
								fill="none"
								stroke="currentColor"
								stroke-width="2"
								style="color: var(--border-primary);"
							>
								<line x1="2" y1="6" x2="10" y2="6"></line>
								<line x1="6" y1="2" x2="6" y2="10"></line>
							</svg>
						</div>
						<span
							style="color: var(--text-primary); font-style: italic; overflow: hidden; text-overflow: ellipsis; white-space: nowrap; flex: 1;"
							title={currentTemplateName || 'Untitled Template'}
						>
							{currentTemplateName || 'Untitled Template'}
						</span>
					</li>
				{/if}
				{#each templates as template (template.id)}
					<!-- svelte-ignore a11y_no_noninteractive_element_to_interactive_role -->
					<li
						class="flex min-w-0 cursor-pointer items-center gap-3 px-4 py-3 hover:opacity-80"
						style={!isNewTemplate && currentTemplateName === template.name
							? 'background-color: var(--bg-secondary);'
							: ''}
						onclick={() => onSelectTemplate(template.name)}
						onkeydown={(e) => e.key === 'Enter' && onSelectTemplate(template.name)}
						role="button"
						tabindex="0"
					>
						<div
							class="flex h-5 w-5 items-center justify-center border-2"
							style="border-color: var(--border-primary);"
						>
							{#if !isNewTemplate && currentTemplateName === template.name}
								<svg
									width="12"
									height="12"
									viewBox="0 0 12 12"
									fill="none"
									stroke="currentColor"
									stroke-width="2"
									style="color: var(--border-primary);"
								>
									<line x1="2" y1="6" x2="10" y2="6"></line>
									<line x1="6" y1="2" x2="6" y2="10"></line>
								</svg>
							{/if}
						</div>
						<span
							style="color: var(--text-primary); overflow: hidden; text-overflow: ellipsis; white-space: nowrap; flex: 1;"
							title={template.name}>{template.name}</span
						>
					</li>
				{/each}
			</ul>
		</div>
	</div>
</div>

<style>
	.btn-create {
		background-color: var(--create-button);
		transition: background-color 0.15s ease;
	}
	.btn-default {
		background-color: var(--create-button);
		transition: background-color 0.15s ease;
	}
	.btn-create:hover {
		background-color: var(--create-button-hover);
	}
	.btn-default:hover:enabled {
		background-color: var(--create-button-hover);
	}
	.btn-create:active {
		background-color: var(--create-button-active);
	}
	.btn-default:disabled {
		opacity: 0.6;
		cursor: not-allowed;
	}
	@media (prefers-color-scheme: dark) {
		.btn-create:hover {
			background-color: #439c43;
		}
		.btn-default:hover:enabled {
			background-color: #3b747c;
		}
		.btn-default:hover:enabled {
			background-color: #3b747c;
		}
	}
</style>
