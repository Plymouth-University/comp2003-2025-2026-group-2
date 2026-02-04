<script lang="ts">
	import type { components } from '$lib/api-types';

	type TemplateVersionInfo = components['schemas']['TemplateVersionInfo'];

	let {
		isOpen = false,
		versions = [],
		currentVersion = 1,
		currentVersionName = null,
		onClose,
		onRestore
	}: {
		isOpen: boolean;
		versions: TemplateVersionInfo[];
		currentVersion?: number;
		currentVersionName?: string | null;
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
				{#if currentVersion && versions.length === 0}
					<div class="mb-4 rounded-lg bg-blue-50 p-4 dark:bg-blue-900/20">
						<div class="flex items-center">
							<div class="flex-shrink-0">
								<svg class="h-5 w-5 text-blue-400" fill="currentColor" viewBox="0 0 20 20">
									<path
										fill-rule="evenodd"
										d="M18 10a8 8 0 11-16 0 8 8 0 0116 0zm-7-4a1 1 0 11-2 0 1 1 0 012 0zM9 9a1 1 0 000 2v3a1 1 0 001 1h1a1 1 0 100-2v-3a1 1 0 00-1-1H9z"
										clip-rule="evenodd"
									/>
								</svg>
							</div>
							<div class="ml-3">
								<h3 class="text-sm font-medium text-blue-800 dark:text-blue-200">
									Current Version: v{currentVersion}
									{#if currentVersionName}
										<span class="font-normal">- {currentVersionName}</span>
									{/if}
								</h3>
								<p class="mt-1 text-xs text-blue-700 dark:text-blue-300">
									No previous versions available yet. Save changes to create version history.
								</p>
							</div>
						</div>
					</div>
				{:else if versions.length === 0}
					<p class="py-8 text-center text-gray-500 dark:text-gray-400">
						No history available for this template.
					</p>
				{:else}
					<div class="mb-4 rounded-lg bg-blue-50 p-3 dark:bg-blue-900/20">
						<div class="flex items-center justify-between">
							<div>
								<span class="text-sm font-semibold text-blue-800 dark:text-blue-200">
									Current Version: v{currentVersion}
								</span>
								{#if currentVersionName}
									<span class="ml-2 text-sm text-blue-700 dark:text-blue-300">
										{currentVersionName}
									</span>
								{/if}
							</div>
							<span
								class="rounded-full bg-blue-600 px-3 py-1 text-xs font-medium text-white dark:bg-blue-500"
							>
								Active
							</span>
						</div>
					</div>
					<h3 class="mb-2 text-sm font-medium text-gray-700 dark:text-gray-300">
						Previous Versions
					</h3>
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
