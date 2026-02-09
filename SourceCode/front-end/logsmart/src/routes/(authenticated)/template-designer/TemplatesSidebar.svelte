<script lang="ts">
	import type { Template } from './types';

	let {
		templates,
		onCreateNew,
		onSelectTemplate,
		currentTemplateName = '',
		isNewTemplate = false
	}: {
		templates: Template[];
		onCreateNew: () => void;
		onSelectTemplate: (templateName: string) => void;
		currentTemplateName?: string;
		isNewTemplate?: boolean;
	} = $props();
</script>

<div
	class="flex flex-col"
	style="border-color: var(--border-primary); background-color: var(--bg-primary);"
>
	<div class="p-6">
		<h2 class="mb-4 text-2xl font-bold" style="color: var(--text-primary);">Templates</h2>

		<button
			class="btn-create mb-6 w-full rounded px-4 py-2 font-medium text-white"
			onclick={onCreateNew}
		>
			+ Create New
		</button>

		<div class="border-2" style="border-color: var(--border-primary);">
			<ul class="divide-y" style="border-color: var(--border-secondary);">
				{#if isNewTemplate}
					<li
						class="flex items-center gap-3 px-4 py-3 min-w-0"
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
					<span style="color: var(--text-primary); font-style: italic; overflow: hidden; text-overflow: ellipsis; white-space: nowrap; flex: 1;" title="{currentTemplateName || 'Untitled Template'}">
							{currentTemplateName || 'Untitled Template'}
						</span>
					</li>
				{/if}
				{#each templates as template (template.id)}
					<!-- svelte-ignore a11y_no_noninteractive_element_to_interactive_role -->
					<li
						class="flex cursor-pointer items-center gap-3 px-4 py-3 hover:opacity-80 min-w-0"
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
						<span style="color: var(--text-primary); overflow: hidden; text-overflow: ellipsis; white-space: nowrap; flex: 1;" title="{template.name}">{template.name}</span>
					</li>
				{/each}
			</ul>
		</div>
	</div>
</div>

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
</style>
