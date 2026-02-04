<script lang="ts">
	import type { components } from '$lib/api-types';

	type TemplateVersionInfo = components['schemas']['TemplateVersionInfo'];

	let {
		isOpen = false,
		versions = [],
		onClose,
		onRestore
	}: {
		isOpen: boolean;
		versions: TemplateVersionInfo[];
		onClose: () => void;
		onRestore: (version: number) => void;
	} = $props();

	function formatDate(dateString: string) {
		return new Date(dateString).toLocaleString();
	}
</script>

{#if isOpen}
	<div
		class="fixed inset-0 z-50 flex items-center justify-center bg-black/50"
		role="dialog"
		aria-modal="true"
	>
		<div class="w-full max-w-2xl rounded-lg bg-white p-6 shadow-xl dark:bg-gray-800">
			<div class="mb-4 flex items-center justify-between">
				<h2 class="text-xl font-bold text-gray-900 dark:text-white">Version History</h2>
				<button
					onclick={onClose}
					class="text-gray-500 hover:text-gray-700 dark:text-gray-400 dark:hover:text-gray-200"
				>
					<span class="sr-only">Close</span>
					<svg class="h-6 w-6" fill="none" viewBox="0 0 24 24" stroke="currentColor">
						<path
							stroke-linecap="round"
							stroke-linejoin="round"
							stroke-width="2"
							d="M6 18L18 6M6 6l12 12"
						/>
					</svg>
				</button>
			</div>

			<div class="max-h-[60vh] overflow-y-auto">
				{#if versions.length === 0}
					<p class="py-8 text-center text-gray-500 dark:text-gray-400">
						No history available for this template.
					</p>
				{:else}
					<table class="min-w-full divide-y divide-gray-200 dark:divide-gray-700">
						<thead class="bg-gray-50 dark:bg-gray-700">
							<tr>
								<th
									scope="col"
									class="px-6 py-3 text-left text-xs font-medium tracking-wider text-gray-500 uppercase dark:text-gray-300"
								>
									Version
								</th>
								<th
									scope="col"
									class="px-6 py-3 text-left text-xs font-medium tracking-wider text-gray-500 uppercase dark:text-gray-300"
								>
									Date
								</th>
								<th
									scope="col"
									class="px-6 py-3 text-left text-xs font-medium tracking-wider text-gray-500 uppercase dark:text-gray-300"
								>
									Name
								</th>
								<th scope="col" class="relative px-6 py-3">
									<span class="sr-only">Restore</span>
								</th>
							</tr>
						</thead>
						<tbody class="divide-y divide-gray-200 bg-white dark:divide-gray-700 dark:bg-gray-800">
							{#each versions as version}
								<tr>
									<td
										class="px-6 py-4 text-sm font-medium whitespace-nowrap text-gray-900 dark:text-white"
									>
										v{version.version}
									</td>
									<td class="px-6 py-4 text-sm whitespace-nowrap text-gray-500 dark:text-gray-300">
										{formatDate(version.created_at)}
									</td>
									<td class="px-6 py-4 text-sm whitespace-nowrap text-gray-500 dark:text-gray-300">
										{version.version_name || '-'}
									</td>
									<td class="px-6 py-4 text-right text-sm font-medium whitespace-nowrap">
										<button
											onclick={() => onRestore(version.version)}
											class="text-indigo-600 hover:text-indigo-900 dark:text-indigo-400 dark:hover:text-indigo-300"
										>
											Restore
										</button>
									</td>
								</tr>
							{/each}
						</tbody>
					</table>
				{/if}
			</div>

			<div class="mt-6 flex justify-end">
				<button
					onclick={onClose}
					class="rounded-md border border-gray-300 bg-white px-4 py-2 text-base font-medium text-gray-700 shadow-sm hover:bg-gray-50 focus:ring-2 focus:ring-indigo-500 focus:ring-offset-2 focus:outline-none dark:border-gray-600 dark:bg-gray-700 dark:text-gray-200 dark:hover:bg-gray-600"
				>
					Close
				</button>
			</div>
		</div>
	</div>
{/if}
