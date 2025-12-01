<script lang="ts">
	import type { Template } from './types';

	let { templates, onCreateNew }: { templates: Template[]; onCreateNew: () => void } = $props();
</script>

<div
	class="w-64 border-r-2"
	style="border-color: var(--border-primary); background-color: var(--bg-primary);"
>
	<div class="p-6">
		<h2 class="mb-4 text-2xl font-bold" style="color: var(--text-primary);">Templates</h2>

		<button
			class="mb-6 w-full rounded px-4 py-2 font-medium text-white"
			style="background-color: #5CB85C;"
			onclick={onCreateNew}
		>
			+ Create New
		</button>

		<div class="border-2" style="border-color: var(--border-primary);">
			<ul class="divide-y" style="border-color: var(--border-secondary);">
				{#each templates as template (template.id)}
					<li class="flex items-center gap-3 px-4 py-3 hover:opacity-80">
						<div
							class="flex h-5 w-5 items-center justify-center border-2"
							style="border-color: var(--border-primary);"
						>
							{#if template.selected}
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
						<span style="color: var(--text-primary);">{template.name}</span>
					</li>
				{/each}
			</ul>
		</div>
	</div>
</div>
