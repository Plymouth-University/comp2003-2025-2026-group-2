<script lang="ts">
	import { toasts, type Toast } from '$lib/stores/toast';
	import { onDestroy } from 'svelte';
	import { SvelteMap } from 'svelte/reactivity';

	const typeStyles: Record<string, { outline: string; accent: string; icon: string }> = {
		success: {
			outline: 'border-l-[4px] border-[var(--create-button)]',
			accent: 'bg-[var(--create-button)]',
			icon: 'M9 12l2 2 4-4m6 2a9 9 0 11-18 0 9 9 0 0118 0z'
		},
		error: {
			outline: 'border-l-[4px] border-[var(--button-secondary)]',
			accent: 'bg-[var(--button-secondary)]',
			icon: 'M10 14l2-2m0 0l2-2m-2 2l-2-2m2 2l2 2m7-2a9 9 0 11-18 0 9 9 0 0118 0z'
		},
		warning: {
			outline: 'border-l-[4px] border-[var(--orange)]',
			accent: 'bg-[var(--orange)]',
			icon: 'M12 9v2m0 4h.01m-6.938 4h13.856c1.54 0 2.502-1.667 1.732-3L13.732 4c-.77-1.333-2.694-1.333-3.464 0L3.34 16c-.77 1.333.192 3 1.732 3z'
		},
		info: {
			outline: 'border-l-[4px] border-[var(--step-active)]',
			accent: 'bg-[var(--step-active)]',
			icon: 'M13 16h-1v-4h-1m1-4h.01M21 12a9 9 0 11-18 0 9 9 0 0118 0z'
		}
	};

	const timers = new SvelteMap<number, ReturnType<typeof setTimeout>>();

	function dismiss(id: number) {
		toasts.remove(id);
		const timer = timers.get(id);
		if (timer) {
			clearTimeout(timer);
			timers.delete(id);
		}
	}

	function scheduleDismiss(toast: Toast) {
		if (typeof window === 'undefined') return;
		const timer = window.setTimeout(() => {
			dismiss(toast.id);
		}, toast.duration);
		timers.set(toast.id, timer);
	}

	function handleToastAdd(currentToasts: Toast[], newToasts: Toast[]) {
		const currentIds = new Set(currentToasts.map((t) => t.id));
		const newIds = new Set(newToasts.map((t) => t.id));

		currentToasts.forEach((toast) => {
			if (!newIds.has(toast.id)) {
				const timer = timers.get(toast.id);
				if (timer) {
					clearTimeout(timer);
					timers.delete(toast.id);
				}
			}
		});

		newToasts.forEach((toast) => {
			if (!currentIds.has(toast.id)) {
				scheduleDismiss(toast);
			}
		});
	}

	let previousToasts: Toast[] = [];
	const unsubscribe = toasts.subscribe((currentToasts) => {
		handleToastAdd(previousToasts, currentToasts);
		previousToasts = currentToasts;
	});

	onDestroy(() => {
		unsubscribe();
		timers.forEach((timer) => clearTimeout(timer));
		timers.clear();
	});
</script>

<div class="fixed right-4 bottom-4 z-50 flex max-w-sm flex-col gap-2">
	{#each $toasts as toast (toast.id)}
		{@const style = typeStyles[toast.type] || typeStyles.info}
		<div class="w-full overflow-hidden rounded-lg border bg-bg-secondary {style.outline} shadow-lg">
			<div class="flex items-start gap-3 p-3">
				<svg
					class="mt-0.5 h-5 w-5 flex-shrink-0"
					fill="none"
					stroke="currentColor"
					viewBox="0 0 24 24"
					style="color: {toast.type === 'success'
						? 'var(--create-button)'
						: toast.type === 'error'
							? 'var(--button-secondary)'
							: toast.type === 'warning'
								? 'var(--orange)'
								: 'var(--step-active)'}"
				>
					<path stroke-linecap="round" stroke-linejoin="round" stroke-width="2" d={style.icon} />
				</svg>
				<div class="flex-1">
					<p class="text-sm font-semibold text-text-primary">{toast.title}</p>
					{#if toast.details.length > 0}
						<div class="mt-1 space-y-0.5 text-xs text-text-secondary">
							{#each toast.details as detail (detail)}
								<p>{detail}</p>
							{/each}
						</div>
					{/if}
				</div>
				<button
					type="button"
					onclick={() => dismiss(toast.id)}
					class="flex-shrink-0 rounded p-1 text-text-secondary hover:bg-bg-primary hover:text-text-primary"
				>
					<svg class="h-4 w-4" fill="none" stroke="currentColor" viewBox="0 0 24 24">
						<path
							stroke-linecap="round"
							stroke-linejoin="round"
							stroke-width="2"
							d="M6 18L18 6M6 6l12 12"
						/>
					</svg>
				</button>
			</div>
			<div
				class="h-1 {style.accent} animate-progress"
				style="animation-duration: {toast.duration}ms"
			></div>
		</div>
	{/each}
</div>

<style>
	@keyframes progress {
		from {
			transform: scaleX(1);
		}
		to {
			transform: scaleX(0);
		}
	}
	.animate-progress {
		transform-origin: left;
		animation-name: progress;
		animation-timing-function: linear;
		animation-fill-mode: forwards;
	}
</style>
