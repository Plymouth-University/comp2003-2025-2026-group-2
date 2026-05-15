<script lang="ts">
	interface Props {
		value: string;
		onchange: (value: string) => void;
	}

	let { value, onchange }: Props = $props();
	let isOpen = $state(false);
	let buttonElement: HTMLButtonElement | undefined = $state();
	let buttonRect = $state<DOMRect | null>(null);

	const options = [
		{ value: 'all', label: 'All Schedules' },
		{ value: 'daily', label: 'Daily' },
		{ value: 'weekly', label: 'Weekly' },
		{ value: 'monthly', label: 'Monthly' },
		{ value: 'quarterly', label: 'Quarterly' },
		{ value: 'yearly', label: 'Yearly' },
		{ value: 'custom', label: 'Custom' }
	];

	function handleSelect(optionValue: string) {
		onchange(optionValue);
		isOpen = false;
	}

	function toggleOpen(e: Event) {
		e.stopPropagation();
		if (buttonElement) {
			buttonRect = buttonElement.getBoundingClientRect();
		}
		isOpen = !isOpen;
	}

	function handleEscape(event: KeyboardEvent) {
		if (event.key === 'Escape') {
			isOpen = false;
		}
	}

	function closeMenu() {
		isOpen = false;
	}
</script>

<svelte:window
	on:click={() => {
		if (isOpen) closeMenu();
	}}
	on:keydown={handleEscape}
/>

<button
	bind:this={buttonElement}
	type="button"
	onclick={toggleOpen}
	class="filter-button"
	aria-label="Open schedule filter"
	aria-expanded={isOpen}
>
	<svg
		width="20"
		height="20"
		viewBox="0 0 24 24"
		fill="none"
		stroke="currentColor"
		stroke-width="2"
		stroke-linecap="round"
		stroke-linejoin="round"
	>
		<polygon points="22 3 2 3 10 12.46 10 19 14 21 14 12.46 22 3"></polygon>
	</svg>
</button>

{#if isOpen && buttonRect}
	<div
		class="filter-dropdown"
		role="menu"
		tabindex="0"
		onclick={(e) => e.stopPropagation()}
		onkeydown={handleEscape}
		style="top: {buttonRect.bottom + 8}px; right: {window.innerWidth - buttonRect.right}px;"
	>
		{#each options as option (option.value)}
			<button
				type="button"
				onclick={() => handleSelect(option.value)}
				class="filter-option"
				class:active={value === option.value}
				role="menuitem"
				tabindex="-1"
			>
				<span class="option-label">{option.label}</span>
				{#if value === option.value}
					<svg
						class="checkmark"
						width="16"
						height="16"
						viewBox="0 0 24 24"
						fill="none"
						stroke="currentColor"
						stroke-width="3"
						stroke-linecap="round"
						stroke-linejoin="round"
					>
						<polyline points="20 6 9 17 4 12"></polyline>
					</svg>
				{/if}
			</button>
		{/each}
	</div>
{/if}

<style>
	.filter-button {
		display: inline-flex;
		align-items: center;
		justify-content: center;
		width: 40px;
		height: 40px;
		min-width: 40px;
		min-height: 40px;
		border: none;
		background: none;
		color: var(--text-primary);
		cursor: pointer;
		border-radius: 6px;
		transition:
			background-color 0.15s ease,
			color 0.15s ease;
		padding: 0;
		flex-shrink: 0;
	}

	.filter-button svg {
		stroke: var(--text-secondary);
		transition: stroke 0.15s ease;
	}

	.filter-button:hover {
		background-color: var(--bg-secondary);
	}

	.filter-button:hover svg {
		stroke: var(--text-primary);
	}

	.filter-button:active {
		background-color: var(--color-border-secondary);
	}

	.filter-button[aria-expanded='true'] {
		background-color: var(--color-border-secondary);
	}

	.filter-button[aria-expanded='true'] svg {
		stroke: var(--text-primary);
	}

	.filter-dropdown {
		position: fixed;
		background-color: var(--bg-primary);
		border: 2px solid var(--border-primary);
		border-radius: 8px;
		box-shadow:
			0 10px 15px -3px rgba(0, 0, 0, 0.1),
			0 4px 6px -2px rgba(0, 0, 0, 0.05);
		z-index: 10000;
		overflow: hidden;
		animation: slideIn 0.15s ease-out;
		min-width: 200px;
	}

	@keyframes slideIn {
		from {
			opacity: 0;
			transform: translateY(-4px);
		}
		to {
			opacity: 1;
			transform: translateY(0);
		}
	}

	.filter-option {
		display: flex;
		align-items: center;
		justify-content: space-between;
		width: 100%;
		padding: 12px 16px;
		border: none;
		background-color: var(--bg-primary);
		color: var(--text-primary);
		cursor: pointer;
		text-align: left;
		transition: background-color 0.12s ease;
		border-bottom: 1px solid var(--border-primary);
		font-size: 14px;
		font-weight: 500;
	}

	.filter-option:last-child {
		border-bottom: none;
	}

	.filter-option:hover {
		background-color: var(--bg-secondary);
	}

	.filter-option:active {
		background-color: var(--color-border-secondary);
	}

	.option-label {
		flex: 1;
		font-weight: 500;
	}

	.checkmark {
		flex-shrink: 0;
		margin-left: 8px;
		color: var(--create-button);
		animation: fadeIn 0.15s ease-out;
	}

	@keyframes fadeIn {
		from {
			opacity: 0;
		}
		to {
			opacity: 1;
		}
	}
</style>
