<script lang="ts">
	interface ClockEvent {
		id: string;
		user_id: string;
		clock_in: string;
		clock_out: string | null;
		status: string;
		created_at: string;
	}

	interface ClockStatus {
		is_clocked_in: boolean;
		current_event: ClockEvent | null;
		recent_events: ClockEvent[];
	}

	let {
		initialStatus = null
	}: {
		initialStatus: ClockStatus | null;
	} = $props();

	let isClockedIn = $state(false);
	let currentEvent = $state<ClockEvent | null>(null);
	let recentEvents = $state<ClockEvent[]>([]);
	let loading = $state(false);
	let error = $state('');
	let initialized = $state(false);

	$effect(() => {
		if (!initialized && initialStatus) {
			isClockedIn = initialStatus.is_clocked_in;
			currentEvent = initialStatus.current_event;
			recentEvents = initialStatus.recent_events;
			initialized = true;
		}
	});

	function formatDateTime(dateStr: string): string {
		const d = new Date(dateStr);
		return d.toLocaleString('en-GB', {
			day: '2-digit',
			month: 'short',
			year: 'numeric',
			hour: '2-digit',
			minute: '2-digit',
			second: '2-digit'
		});
	}

	function formatDuration(clockIn: string, clockOut: string | null): string {
		if (!clockOut) return 'Ongoing';
		const start = new Date(clockIn).getTime();
		const end = new Date(clockOut).getTime();
		const diffMs = end - start;
		const hours = Math.floor(diffMs / (1000 * 60 * 60));
		const minutes = Math.floor((diffMs % (1000 * 60 * 60)) / (1000 * 60));
		return `${hours}h ${minutes}m`;
	}

	async function refreshStatus() {
		try {
			const res = await fetch('/api/clock/status');
			if (res.ok) {
				const data: ClockStatus = await res.json();
				isClockedIn = data.is_clocked_in;
				currentEvent = data.current_event;
				recentEvents = data.recent_events;
			}
		} catch (e) {
			console.error('Failed to refresh clock status:', e);
		}
	}

	async function handleClockIn() {
		loading = true;
		error = '';
		try {
			const res = await fetch('/api/clock/in', { method: 'POST' });
			if (res.ok) {
				await refreshStatus();
			} else {
				const data = await res.json();
				error = data.error || 'Failed to clock in';
			}
		} catch {
			error = 'Network error. Please try again.';
		} finally {
			loading = false;
		}
	}

	async function handleClockOut() {
		loading = true;
		error = '';
		try {
			const res = await fetch('/api/clock/out', { method: 'POST' });
			if (res.ok) {
				await refreshStatus();
			} else {
				const data = await res.json();
				error = data.error || 'Failed to clock out';
			}
		} catch {
			error = 'Network error. Please try again.';
		} finally {
			loading = false;
		}
	}
</script>

<div class="flex h-full w-full flex-col border-2" style="border-color: var(--border-primary);">
	<div
		class="border-b-2 px-6 py-4"
		style="border-color: var(--border-primary); background-color: var(--bg-primary);"
	>
		<h2 class="text-xl font-bold" style="color: var(--text-primary);">Clock In / Out</h2>
	</div>
	<div class="flex-1 overflow-auto px-6 py-6" style="background-color: var(--bg-primary);">
		<!-- Current Status -->
		<div class="mb-4 flex items-center gap-3">
			<span
				class="inline-block h-3 w-3 rounded-full"
				style="background-color: {isClockedIn ? '#22c55e' : '#ef4444'};"
			></span>
			<span class="text-base font-medium" style="color: var(--text-primary);">
				{isClockedIn ? 'Currently Clocked In' : 'Currently Clocked Out'}
			</span>
		</div>

		{#if isClockedIn && currentEvent}
			<div class="mb-4 text-sm" style="color: var(--text-secondary);">
				Since: {formatDateTime(currentEvent.clock_in)}
			</div>
		{/if}

		<!-- Error message -->
		{#if error}
			<div class="mb-4 text-sm font-medium" style="color: #ef4444;">
				{error}
			</div>
		{/if}

		<!-- Clock In / Out Button -->
		<div class="mb-6">
			{#if isClockedIn}
				<button
					onclick={handleClockOut}
					disabled={loading}
					class="cursor-pointer border-2 px-6 py-2 font-medium transition-opacity hover:opacity-80 disabled:cursor-not-allowed disabled:opacity-50"
					style="border-color: #ef4444; color: #ef4444; background-color: var(--bg-primary);"
				>
					{loading ? 'Processing...' : 'Clock Out'}
				</button>
			{:else}
				<button
					onclick={handleClockIn}
					disabled={loading}
					class="cursor-pointer border-2 px-6 py-2 font-medium transition-opacity hover:opacity-80 disabled:cursor-not-allowed disabled:opacity-50"
					style="border-color: #22c55e; color: #22c55e; background-color: var(--bg-primary);"
				>
					{loading ? 'Processing...' : 'Clock In'}
				</button>
			{/if}
		</div>

		<!-- Recent Events -->
		{#if recentEvents.length > 0}
			<div>
				<h3 class="mb-3 text-sm font-semibold" style="color: var(--text-primary);">
					Recent Activity
				</h3>
				<div class="space-y-2">
					{#each recentEvents as event}
						<div
							class="flex items-center justify-between gap-4 border-b py-2 text-sm"
							style="border-color: var(--border-primary);"
						>
							<div class="flex items-center gap-2">
								<span
									class="inline-block h-2 w-2 rounded-full"
									style="background-color: {event.status === 'in' ? '#22c55e' : '#ef4444'};"
								></span>
								<span style="color: var(--text-primary);">
									{formatDateTime(event.clock_in)}
								</span>
							</div>
							<div style="color: var(--text-secondary);">
								{#if event.clock_out}
									→ {formatDateTime(event.clock_out)}
								{:else}
									<span style="color: #22c55e;">Active</span>
								{/if}
							</div>
							<div class="font-medium" style="color: var(--text-secondary);">
								{formatDuration(event.clock_in, event.clock_out)}
							</div>
						</div>
					{/each}
				</div>
			</div>
		{:else}
			<div style="color: var(--text-secondary);">No clock events yet</div>
		{/if}
	</div>
</div>
