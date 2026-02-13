<script lang="ts">
	import type { PageData } from './$types';

	let { data } = $props<{ data: PageData }>();

	const isMember = $derived(data?.user?.role === 'member');
	const isAdmin = $derived(data?.user?.role === 'admin' || data?.user?.role === 'logsmart_admin');

	const sortedPastLogs = $derived(
		data.pastLogs
			? [...data.pastLogs].sort((a, b) => {
					const dateA = new Date(a.period);
					const dateB = new Date(b.period);
					return dateB.getTime() - dateA.getTime();
				})
			: []
	);

	const dueTodayTemplateNames = $derived(
		new Set(data.dueToday?.map((form: any) => form.template_name) || [])
	);

	const sortedAllLogs = $derived(
		data.allLogs
			? [...data.allLogs]
					.filter((log) => !dueTodayTemplateNames.has(log.template_name))
					.sort((a, b) => {
						const dateA = new Date(a.period);
						const dateB = new Date(b.period);
						return dateB.getTime() - dateA.getTime();
					})
			: []
	);

	function formatDate(dateString: string): string {
		const date = new Date(dateString);
		const now = new Date();
		const today = new Date(now.getFullYear(), now.getMonth(), now.getDate());
		const yesterday = new Date(today);
		yesterday.setDate(yesterday.getDate() - 1);
		const dateOnly = new Date(date.getFullYear(), date.getMonth(), date.getDate());

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
		const date = new Date(dateString);
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
				(log: any) =>
					log.template_name === templateName && log.period === period && log.status === 'draft'
			);

			if (draftEntry) {
				window.location.href = `/log-template?entry=${encodeURIComponent(draftEntry.id)}&mode=edit`;
				return;
			}
		}

		window.location.href = `/log-template?template=${encodeURIComponent(templateName)}`;
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

	function handleEditLog(entryId: string) {
		window.location.href = `/log-template?entry=${encodeURIComponent(entryId)}&mode=edit`;
	}
</script>

<svelte:head>
	<title>Logs List</title>
</svelte:head>
<main>
	<div class="min-h-screen" style="background-color: var(--bg-secondary);">
		<div class="mx-auto max-w-7xl px-6 py-8">
			{#if data.error}
				<div
					class="mb-4 rounded p-4"
					style="background-color: #fee; border: 1px solid #fcc; color: #c00;"
				>
					{data.error}
				</div>
			{/if}

			{#if isMember}
				<div class="mb-8">
					<h2 class="mb-4 text-3xl font-bold" style="color: var(--text-primary);">
						Logs Due Today
					</h2>
					{#if data.dueToday && data.dueToday.length > 0}
						<div class="space-y-2">
							{#each data.dueToday as form}
								<div
									class="flex items-center justify-between rounded border-2 p-4"
									style="background-color: var(--bg-primary); border-color: var(--border-primary);"
								>
									<div class="min-w-0 flex-1">
										<div
											class="text-overflow-ellipsis overflow-hidden text-xl font-semibold whitespace-nowrap"
											style="color: var(--text-primary);"
											title={formatTemplateName(form.template_name, form.period)}
										>
											{formatTemplateName(form.template_name, form.period)}
										</div>
										<div class="text-sm" style="color: var(--text-secondary);">
											{#if form.status}
												Status: {form.status}
												{#if form.last_submitted}
													| Last submitted: <span title={formatFullDateTime(form.last_submitted)}
														>{formatDate(form.last_submitted)}</span
													>
												{/if}
											{:else}
												Not yet started
											{/if}
										</div>
									</div>
									<button
										onclick={() => handleFillLog(form.template_name, form.period, form.status)}
										class="rounded px-6 py-2 font-semibold hover:opacity-80"
										style="background-color: #3D7A82; color: white;"
									>
										Fill Out
									</button>
								</div>
							{/each}
						</div>
					{:else}
						<div
							class="rounded p-6 text-center"
							style="background-color: var(--bg-primary); color: var(--text-secondary);"
						>
							No logs due today
						</div>
					{/if}
				</div>

				<div>
					<h2 class="mb-4 text-3xl font-bold" style="color: var(--text-primary);">Past Logs</h2>
					{#if sortedPastLogs && sortedPastLogs.length > 0}
						<div class="space-y-2">
							{#each sortedPastLogs as log}
								<div
									class="flex items-center justify-between rounded border-2 p-4"
									style="background-color: var(--bg-primary); border-color: var(--border-primary);"
								>
									<div class="min-w-0 flex-1">
										<div
											class="text-overflow-ellipsis overflow-hidden text-xl font-semibold whitespace-nowrap"
											style="color: var(--text-primary);"
											title={formatTemplateName(log.template_name, log.period)}
										>
											{formatTemplateName(log.template_name, log.period)}
										</div>
										<div class="text-sm" style="color: var(--text-secondary);">
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
									</div>
									<button
										onclick={() => handleViewLog(log.id)}
										class="rounded px-6 py-2 hover:opacity-80"
										style="background-color: var(--bg-secondary); color: var(--text-primary);"
									>
										View
									</button>
								</div>
							{/each}
						</div>
					{:else}
						<div
							class="rounded p-6 text-center"
							style="background-color: var(--bg-primary); color: var(--text-secondary);"
						>
							No past logs found
						</div>
					{/if}
				</div>
			{:else if isAdmin}
				<div class="mb-8">
					<h2 class="mb-4 text-3xl font-bold" style="color: var(--text-primary);">
						Logs Due Today
					</h2>
					{#if data.dueToday && data.dueToday.length > 0}
						<div class="space-y-2">
							{#each data.dueToday as form}
								<div
									class="flex items-center justify-between rounded border-2 p-4"
									style="background-color: var(--bg-primary); border-color: var(--border-primary);"
								>
									<div class="min-w-0 flex-1">
										<div
											class="text-overflow-ellipsis overflow-hidden text-xl font-semibold whitespace-nowrap"
											style="color: var(--text-primary);"
											title={formatTemplateName(form.template_name, form.period)}
										>
											{formatTemplateName(form.template_name, form.period)}
										</div>
										<div class="text-sm" style="color: var(--text-secondary);">
											{#if form.status}
												Status: {form.status}
												{#if form.last_submitted}
													| Last submitted: <span title={formatFullDateTime(form.last_submitted)}
														>{formatDate(form.last_submitted)}</span
													>
												{/if}
											{:else}
												Not yet started
											{/if}
										</div>
									</div>
									<button
										onclick={() => handleFillLog(form.template_name, form.period, form.status)}
										class="rounded px-6 py-2 font-semibold hover:opacity-80"
										style="background-color: #3D7A82; color: white;"
									>
										Fill Out
									</button>
								</div>
							{/each}
						</div>
					{:else}
						<div
							class="rounded p-6 text-center"
							style="background-color: var(--bg-primary); color: var(--text-secondary);"
						>
							No logs due today
						</div>
					{/if}
				</div>

				<div>
					<h2 class="mb-4 text-3xl font-bold" style="color: var(--text-primary);">All Logs</h2>
					{#if sortedAllLogs && sortedAllLogs.length > 0}
						<div class="space-y-2">
							{#each sortedAllLogs as log}
								<div
									class="flex items-center justify-between rounded border-2 p-4"
									style="background-color: var(--bg-primary); border-color: var(--border-primary);"
								>
									<div class="min-w-0 flex-1">
										<div
											class="text-overflow-ellipsis overflow-hidden text-xl font-semibold whitespace-nowrap"
											style="color: var(--text-primary);"
											title={formatTemplateName(log.template_name, log.period)}
										>
											{formatTemplateName(log.template_name, log.period)}
										</div>
										<div class="text-sm" style="color: var(--text-secondary);">
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
									</div>
									<div class="flex gap-2">
										{#if log.status === 'submitted'}
											<button
												onclick={() => handleViewLog(log.id)}
												class="rounded px-6 py-2 hover:opacity-80"
												style="background-color: var(--bg-secondary); color: var(--text-primary);"
											>
												View
											</button>
											<button
												onclick={() => handleUnsubmit(log.id)}
												class="rounded px-6 py-2 hover:opacity-80"
												style="background-color: #f59e0b; color: white;"
											>
												Unsubmit
											</button>
										{:else}
											<button
												onclick={() => handleEditLog(log.id)}
												class="rounded px-6 py-2 hover:opacity-80"
												style="background-color: #3D7A82; color: white;"
											>
												Edit
											</button>
										{/if}
									</div>
								</div>
							{/each}
						</div>
					{:else}
						<div
							class="rounded p-6 text-center"
							style="background-color: var(--bg-primary); color: var(--text-secondary);"
						>
							No logs found
						</div>
					{/if}
				</div>
			{/if}
		</div>
	</div>
</main>
