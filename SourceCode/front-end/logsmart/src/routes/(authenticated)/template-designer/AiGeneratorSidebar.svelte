<script lang="ts">
	let {
		aiPrompt = $bindable(''),
		onGenerateLayout,
		onUndoGeneration,
		aiLoading = false,
		aiError = null,
		hasUndoAvailable = false
	}: {
		aiPrompt?: string;
		onGenerateLayout?: () => void;
		onUndoGeneration?: () => void;
		aiLoading?: boolean;
		aiError?: string | null;
		hasUndoAvailable?: boolean;
	} = $props();

	let secondsElapsed = $state(0);
	let timerInterval: ReturnType<typeof setInterval> | null = null;

	$effect(() => {
		if (aiLoading) {
			secondsElapsed = 0;
			timerInterval = setInterval(() => {
				secondsElapsed += 1;
			}, 1000);
		} else {
			if (timerInterval) {
				clearInterval(timerInterval);
				timerInterval = null;
			}
			secondsElapsed = 0;
		}

		return () => {
			if (timerInterval) {
				clearInterval(timerInterval);
			}
		};
	});
</script>

<div
	class="flex flex-col"
	style="border-color: var(--border-primary); background-color: var(--bg-primary);"
>
	<div class="p-4">
		<h2 class="mb-4 text-xl font-bold text-text-primary">AI Generator</h2>

		<div class="mb-4">
			<label for="ai-prompt" class="mb-2 block text-sm font-medium text-text-secondary"
				>Template Prompt</label
			>
			<textarea
				id="ai-prompt"
				bind:value={aiPrompt}
				placeholder="Describe your log template layout..."
				class="w-full rounded border-2 p-3 text-sm"
				style="border-color: var(--border-primary); background-color: var(--bg-secondary); color: var(--text-primary);"
				disabled={aiLoading}
				rows="4"
			></textarea>
		</div>

		<div class="flex gap-2">
			<button
				class="flex-1 rounded px-4 py-2 font-medium text-white disabled:opacity-50"
				style="background-color: #6366f1;"
				onclick={onGenerateLayout}
				disabled={aiLoading || !aiPrompt.trim()}
			>
				{#if aiLoading}
					✨ Generating... ({secondsElapsed}s)
				{:else}
					✨ Generate
				{/if}
			</button>
			{#if hasUndoAvailable}
				<button
					class="rounded px-3 py-2 text-sm font-medium"
					style="background-color: var(--bg-secondary); color: var(--text-primary); border: 1px solid var(--border-primary);"
					onclick={onUndoGeneration}
					disabled={aiLoading}
					title="Undo the last AI generation"
				>
					↶
				</button>
			{/if}
		</div>

		{#if aiError}
			<div class="mt-3 rounded border-l-4 border-red-500 bg-red-50 p-3 text-sm text-red-700">
				{aiError}
			</div>
		{/if}
	</div>
</div>
