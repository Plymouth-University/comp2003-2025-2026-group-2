<script lang="ts">
	import type { PageData } from './$types';
	import type { components } from '$lib/api-types';
	import ClockInOut from '$lib/components/ClockInOut.svelte';
	import LogRow from '$lib/components/logs/LogRow.svelte';
	import LogSection from '$lib/components/logs/LogSection.svelte';
	import { onDestroy, onMount } from 'svelte';
	import { SvelteDate } from 'svelte/reactivity';

	type LogEntry = components['schemas']['LogEntryResponse'];
	type DueForm = components['schemas']['DueFormInfo'];

	function parsePeriodToDate(period: string): Date | null {
		const slashParts = period.split('/');

		if (slashParts.length === 1) {
			const year = parseInt(slashParts[0], 10);
			const date = new Date(year, 0, 1);
			return isNaN(date.getTime()) ? null : date;
		} else if (slashParts.length === 2) {
			const month = parseInt(slashParts[0], 10);
			const year = parseInt(slashParts[1], 10);
			const date = new Date(year, month - 1, 1);
			return isNaN(date.getTime()) ? null : date;
		} else if (slashParts.length === 3) {
			const isWeekly = slashParts[0].includes('-');
			const day = isWeekly
				? parseInt(slashParts[0].split('-')[1], 10)
				: parseInt(slashParts[0], 10);
			const month = parseInt(slashParts[1], 10);
			const year = parseInt(slashParts[2], 10);
			const date = new Date(year, month - 1, day);
			return isNaN(date.getTime()) ? null : date;
		}
		return null;
	}

	let { data } = $props<{ data: PageData }>();

	const dueToday = $derived(data?.dueToday || []) as DueForm[] | undefined;

	const isReadonlyHQ = $derived(data?.user?.role === 'staff' && !data?.user?.branch_id);
	const isStaff = $derived(data?.user?.role === 'staff' && !isReadonlyHQ);
	const isAdmin = $derived(
		data?.user?.role.match(/company_manager|branch_manager|logsmart_admin/) || isReadonlyHQ
	);

	const sortedPastLogs = $derived(
		data.pastLogs
			? [...data.pastLogs].sort((a, b) => {
					const dateA = parsePeriodToDate(a.period);
					const dateB = parsePeriodToDate(b.period);
					if (!dateA || !dateB) return 0;
					return dateB.getTime() - dateA.getTime();
				})
			: []
	);

	const dueTodayPeriods = $derived(
		new Set(dueToday?.map((form: DueForm) => `${form.template_name}|${form.period}`) || [])
	);

	const sortedAllLogs = $derived(
		data.allLogs
			? [...data.allLogs]
					.filter((log) => !dueTodayPeriods.has(`${log.template_name}|${log.period}`))
					.sort((a, b) => {
						const dateA = parsePeriodToDate(a.period);
						const dateB = parsePeriodToDate(b.period);
						if (!dateA || !dateB) return 0;
						return dateB.getTime() - dateA.getTime();
					})
			: []
	);

	let showToast = $state(false);
	let toastType = $state<'success' | 'error'>('success');
	let toastMessage = $state('');
	let toastSequence = $state(0);
	let toastTimer = $state<number | null>(null);
	const TOAST_DURATION_MS = 5000;

	function showTimedToast(
		type: 'success' | 'error',
		message: string,
		durationMs = TOAST_DURATION_MS
	) {
		toastType = type;
		toastMessage = message;
		toastSequence += 1;
		showToast = true;

		if (toastTimer !== null) {
			window.clearTimeout(toastTimer);
		}

		toastTimer = window.setTimeout(() => {
			showToast = false;
			toastTimer = null;
		}, durationMs);
	}

	onMount(() => {
		const params = new URLSearchParams(window.location.search);
		const toastToken = params.get('toast');

		if (toastToken === 'log_submitted_success') {
			showTimedToast('success', 'Log submitted successfully');
			params.delete('toast');
			const nextQuery = params.toString();
			const nextUrl = `${window.location.pathname}${nextQuery ? `?${nextQuery}` : ''}${window.location.hash}`;
			window.history.replaceState(window.history.state, '', nextUrl);
		}
	});

	onDestroy(() => {
		if (toastTimer !== null) {
			window.clearTimeout(toastTimer);
		}
	});

	function formatDate(dateString: string): string {
		const date = new SvelteDate(dateString);
		const now = new SvelteDate();
		const today = new SvelteDate(now.getFullYear(), now.getMonth(), now.getDate());
		const yesterday = new SvelteDate(today);
		yesterday.setDate(yesterday.getDate() - 1);
		const dateOnly = new SvelteDate(date.getFullYear(), date.getMonth(), date.getDate());

		if (dateOnly.getTime() === today.getTime()) {
			return date.toLocaleTimeString('en-GB', {
				hour: '2-digit',
				minute: '2-digit'
			});
		} else if (dateOnly.getTime() === yesterday.getTime()) {
			return 'Yesterday';
		} else {
			return date.toLocaleDateString('en-GB', {
				day: '2-digit',
				month: '2-digit',
				year: 'numeric'
			});
		}
	}

	function formatFullDateTime(dateString: string): string {
		const date = new SvelteDate(dateString);
		return date.toLocaleString('en-GB', {
			day: '2-digit',
			month: '2-digit',
			year: 'numeric',
			hour: '2-digit',
			minute: '2-digit',
			second: '2-digit'
		});
	}

	function formatTemplateName(templateName: string, period?: string): string {
		if (period && templateName.includes('{period}')) {
			return templateName.replace('{period}', period);
		}
		return templateName;
	}

	function handleFillLog(templateName: string, period?: string, status?: string | null) {
		if (status === 'draft' && period) {
			const draftEntry = (data.allLogs || data.pastLogs || []).find(
				(log: LogEntry) =>
					log.template_name === templateName && log.period === period && log.status === 'draft'
			);

			if (draftEntry) {
				window.location.href = `/log-template?entry=${encodeURIComponent(draftEntry.id)}&mode=edit`;
				return;
			}
		}

		let url = `/log-template?template=${encodeURIComponent(templateName)}`;
		if (period) {
			url += `&period=${encodeURIComponent(period)}`;
		}
		window.location.href = url;
	}

	function handleViewLog(entryId: string) {
		window.location.href = `/log-template?entry=${encodeURIComponent(entryId)}`;
	}

	async function handleUnsubmit(entryId: string) {
		if (
			!confirm('Are you sure you want to unsubmit this log? This will allow it to be edited again.')
		) {
			return;
		}

		try {
			const response = await fetch(`/api/logs/entries/${entryId}/unsubmit`, {
				method: 'POST',
				headers: {
					'Content-Type': 'application/json'
				}
			});

			if (response.ok) {
				alert('Log unsubmitted successfully');
				window.location.reload();
			} else {
				const error = await response.text();
				console.error('Unsubmit failed:', error);
				alert('Failed to unsubmit log');
			}
		} catch (error) {
			console.error('Error unsubmitting log:', error);
			alert('Error unsubmitting log');
		}
	}
</script>

{#if showToast}
	{#key toastSequence}
		<div
			class="fixed right-4 bottom-4 z-50 w-full max-w-sm overflow-hidden rounded-lg border px-4 py-3 text-left shadow-lg"
			style={toastType === 'success'
				? 'border-color: #16a34a; background-color: #f0fdf4;'
				: 'border-color: #dc2626; background-color: #fef2f2;'}
		>
			<div class="mb-2 flex items-start justify-between gap-3">
				<p
					class="text-sm font-semibold"
					style={toastType === 'success' ? 'color: #166534;' : 'color: #991b1b;'}
				>
					{toastMessage}
				</p>
				<button
					type="button"
					onclick={() => {
						showToast = false;
						if (toastTimer !== null) {
							window.clearTimeout(toastTimer);
							toastTimer = null;
						}
					}}
					class="rounded px-2 py-0.5 text-xs font-semibold transition-opacity hover:opacity-80"
					style={toastType === 'success'
						? 'background-color: #dcfce7; color: #166534; cursor: pointer;'
						: 'background-color: #fee2e2; color: #991b1b; cursor: pointer;'}
				>
					Close
				</button>
			</div>
			<div
				class="logs-toast-progress absolute right-0 bottom-0 left-0 h-1"
				style={toastType === 'success'
					? `background-color: #16a34a; animation-duration: ${TOAST_DURATION_MS}ms;`
					: `background-color: #dc2626; animation-duration: ${TOAST_DURATION_MS}ms;`}
			></div>
		</div>
	{/key}
{/if}

<svelte:head>
	<title>Logs List</title>
</svelte:head>
<main>
	<div class="min-h-full" style="background-color: var(--bg-secondary);">
		<div class="mx-auto max-w-7xl px-6 py-8">
			{#if data.error}
				<div
					class="mb-4 rounded p-4"
					style="background-color: #fee; border: 1px solid #fcc; color: #c00;"
				>
					{data.error}
				</div>
			{/if}

			{#if isStaff}
				<div class="mb-8">
					<ClockInOut initialStatus={data.clockStatus ?? null} />
				</div>
			{/if}

			{#if isStaff || isAdmin}
				<LogSection
					title="Logs Due Today"
					hasItems={Boolean(dueToday && dueToday.length > 0)}
					emptyMessage="No logs due today"
				>
					{#each dueToday || [] as form (form.template_name + form.period)}
						<LogRow
							title={formatTemplateName(form.template_name, form.period)}
							titleAttr={formatTemplateName(form.template_name, form.period)}
						>
							{#snippet meta()}
								<div>
									Period: {form.period} |
									{#if form.status}
										Status: {form.status}
										{#if form.last_submitted}
											| Last submitted: <span title={formatFullDateTime(form.last_submitted)}
												>{formatDate(form.last_submitted)}</span
											>
										{/if}
									{:else if form.availability_status === 'overdue'}
										Status: overdue
									{:else if form.availability_status === 'not_available'}
										Status: not yet started
									{:else}
										Status: Available
									{/if}
								</div>
							{/snippet}
							{#snippet actions()}
								{#if !isReadonlyHQ}
									<button
										onclick={() => handleFillLog(form.template_name, form.period, form.status)}
										class="rounded px-3 py-2 font-semibold hover:opacity-80 lg:px-6"
										style="background-color: #3D7A82; color: white;"
									>
										Fill Out
									</button>
								{/if}
							{/snippet}
						</LogRow>
					{/each}
				</LogSection>
			{/if}

			{#if isStaff}
				<div>
					<LogSection
						title="Past Logs"
						hasItems={Boolean(sortedPastLogs && sortedPastLogs.length > 0)}
						emptyMessage="No past logs found"
					>
						{#each sortedPastLogs as log (log.id)}
							<LogRow
								title={formatTemplateName(log.template_name, log.period)}
								titleAttr={formatTemplateName(log.template_name, log.period)}
							>
								{#snippet meta()}
									<div>
										Created: <span title={formatFullDateTime(log.created_at)}
											>{formatDate(log.created_at)}</span
										>
										{#if log.submitted_at}
											| Submitted: <span title={formatFullDateTime(log.submitted_at)}
												>{formatDate(log.submitted_at)}</span
											>
										{/if}
										| Status: {log.status}
									</div>
								{/snippet}
								{#snippet actions()}
									<button
										onclick={() => handleViewLog(log.id)}
										class="rounded px-3 py-2 hover:opacity-80 lg:px-6"
										style="background-color: var(--bg-secondary); color: var(--text-primary);"
									>
										View
									</button>
								{/snippet}
							</LogRow>
						{/each}
					</LogSection>
				</div>
			{:else if isAdmin}
				<div>
					<LogSection
						title="All Logs"
						hasItems={Boolean(sortedAllLogs && sortedAllLogs.length > 0)}
						emptyMessage="No logs found"
					>
						{#each sortedAllLogs as log (log.id)}
							<LogRow
								title={formatTemplateName(log.template_name, log.period)}
								titleAttr={formatTemplateName(log.template_name, log.period)}
							>
								{#snippet meta()}
									<div>
										Created: <span title={formatFullDateTime(log.created_at)}
											>{formatDate(log.created_at)}</span
										>
										{#if log.submitted_at}
											| Submitted: <span title={formatFullDateTime(log.submitted_at)}
												>{formatDate(log.submitted_at)}</span
											>
										{/if}
										| Status: {log.status}
									</div>
								{/snippet}
								{#snippet actions()}
									{#if log.status === 'submitted'}
										<button
											onclick={() => handleViewLog(log.id)}
											class="rounded px-3 py-2 hover:opacity-80 lg:px-6"
											style="background-color: var(--bg-secondary); color: var(--text-primary);"
										>
											View
										</button>
										{#if !isReadonlyHQ}
											<button
												onclick={() => handleUnsubmit(log.id)}
												class="rounded px-3 py-2 hover:opacity-80 lg:px-6"
												style="background-color: #f59e0b; color: white;"
											>
												Unsubmit
											</button>
										{/if}
									{/if}
								{/snippet}
							</LogRow>
						{/each}
					</LogSection>
				</div>
			{/if}
		</div>
	</div>
</main>

<style>
	@keyframes logsToastCountdown {
		from {
			transform: scaleX(1);
		}
		to {
			transform: scaleX(0);
		}
	}

	.logs-toast-progress {
		transform-origin: left center;
		animation-name: logsToastCountdown;
		animation-timing-function: linear;
		animation-fill-mode: forwards;
	}
</style>
