<script lang="ts">
	import { onMount } from 'svelte';

	let { open = $bindable(false), title = 'Confirm', message = '', onConfirm, onCancel } = $props<{
		open?: boolean;
		title?: string;
		message?: string;
		onConfirm?: () => void;
		onCancel?: () => void;
	}>();

	function handleConfirm() {
		onConfirm?.();
		open = false;
	}

	function handleCancel() {
		onCancel?.();
		open = false;
	}

	function handleKeydown(event: KeyboardEvent) {
		if (event.key === 'Escape') {
			handleCancel();
		}
	}

	function handleBackdropClick(event: MouseEvent) {
		if (event.target === event.currentTarget) {
			handleCancel();
		}
	}
</script>

<svelte:window onkeydown={handleKeydown} />

{#if open}
	<div
		class="fixed inset-0 z-50 flex items-center justify-center bg-black/50"
		role="dialog"
		aria-modal="true"
		aria-labelledby="confirm-dialog-title"
		onclick={handleBackdropClick}
	>
		<div
			class="w-full max-w-md rounded-lg border-2 border-border-primary bg-bg-primary p-6 shadow-lg"
		>
			<h2 id="confirm-dialog-title" class="mb-4 text-xl font-bold text-text-primary">
				{title}
			</h2>
			<p class="mb-6 text-text-secondary">{message}</p>
			<div class="flex justify-end gap-3">
				<button
					type="button"
					class="rounded border-2 border-border-primary bg-bg-primary px-4 py-2 font-bold text-text-primary hover:opacity-80"
					onclick={handleCancel}
				>
					Cancel
				</button>
				<button
					type="button"
					class="rounded bg-[var(--button-secondary)] px-4 py-2 font-bold text-white hover:opacity-80"
					onclick={handleConfirm}
				>
					Confirm
				</button>
			</div>
		</div>
	</div>
{/if}