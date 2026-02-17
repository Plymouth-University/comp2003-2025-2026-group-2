<script lang="ts">
	import type { PageData } from './$types';
	import { goto } from '$app/navigation';
	import ClockInOut from '$lib/components/ClockInOut.svelte';

	let { data } = $props<{ data: PageData }>();

	const todaysLogs = $derived(data?.todaysLogs ?? []);
	const clockStatus = $derived(data?.clockStatus ?? null);

	const handleCreateNewTemplate = () => {
		goto('/template-designer');
	};

	const handleViewReports = () => {
		goto('/reports');
	};

	// Get user data from server load
	const user = $derived((() => {
		if (!data.user) {
			return {
				name: 'Loading...',
				email: '',
				company: '',
				role: '',
				initials: '?'
			};
		}

		const firstName = data.user.first_name || '';
		const lastName = data.user.last_name || '';
		const fullName = `${firstName} ${lastName}`.trim();
		const initials = (firstName.charAt(0) + lastName.charAt(0)).toUpperCase();
		let role = '';
		switch (data.user.role) {
			case 'logsmart_admin':
				role = 'LogSmart Internal Administrator';
				break;
			case 'staff':
				role = 'Staff Member';
				break;
			case 'company_manager':
				role = 'Company Manager';
				break;
			case 'branch_manager':
				role = 'Branch Manager';
				break;
			default:
				break;
		}

		return {
			name: fullName || 'User',
			email: data.user.email || '',
			company: data.user.company_name || 'N/A',
			role: role,
			initials: initials || '?'
		};
	})());

	// Draggable boxes state
	type BoxId = 'logs' | 'clock' | 'actions';
	let boxes = $state<BoxId[]>(['logs', 'clock', 'actions']);
	let draggedBox: BoxId | null = $state(null);
	let dragOverIndex: number | null = $state(null);
	let mounted = $state(false);

	// Load saved box order from localStorage after mount
	$effect(() => {
		if (typeof window !== 'undefined') {
			const saved = localStorage.getItem('dashboard-box-order');
			if (saved) {
				try {
					const parsed = JSON.parse(saved) as BoxId[];
					if (Array.isArray(parsed) && parsed.length === 3) {
						boxes = parsed;
					}
				} catch {
					// Ignore invalid data
				}
			}
			mounted = true;
		}
	});

	// Save box order to localStorage whenever it changes
	$effect(() => {
		if (typeof window !== 'undefined' && boxes.length === 3 && mounted) {
			localStorage.setItem('dashboard-box-order', JSON.stringify(boxes));
		}
	});

	function handleDragStart(boxId: BoxId) {
		draggedBox = boxId;
	}

	function handleDragEnd() {
		draggedBox = null;
		dragOverIndex = null;
	}

	function handleDragOver(event: DragEvent, index: number) {
		event.preventDefault();
		dragOverIndex = index;
	}

	function handleDrop(event: DragEvent, targetIndex: number) {
		event.preventDefault();
		if (draggedBox === null) return;

		const draggedIndex = boxes.indexOf(draggedBox);
		if (draggedIndex === -1) return;

		// Reorder the boxes array
		const newBoxes = [...boxes];
		newBoxes.splice(draggedIndex, 1);
		newBoxes.splice(targetIndex, 0, draggedBox);
		boxes = newBoxes;

		draggedBox = null;
		dragOverIndex = null;
	}
</script>

<svelte:head>
	<title>Dashboard</title>
</svelte:head>
<div class="h-full w-full overflow-auto" style="background-color: var(--bg-secondary);">
	<!-- Main Content -->
	<div class="mx-auto max-w-450 px-6 py-8">
		<!-- Header with User Profile -->
		<div class="mb-6 flex flex-wrap items-start justify-between gap-4">
			<div>
				<h1 class="text-3xl font-bold" style="color: var(--text-primary);">Dashboard Overview</h1>
			</div>

			<!-- User Profile Section -->
			<div class="border-2" style="border-color: var(--border-primary);">
				<div class="px-6 py-4" style="background-color: var(--bg-primary);">
					<div class="flex items-center gap-4">
						<!-- Profile Picture (Initials) -->
						<div
							class="flex h-16 w-16 shrink-0 items-center justify-center rounded-full text-xl font-bold text-white"
							style="background-color: #3D7A82;"
						>
							{user.initials}
						</div>
						<!-- User Info -->
						<div class="text-left">
							<div class="font-bold" style="color: var(--text-primary);">{user.name}</div>
							<div class="text-sm" style="color: var(--text-secondary);">{user.email}</div>
							<div class="text-sm" style="color: var(--text-secondary);">{user.company}</div>
							<div class="text-sm font-medium" style="color: var(--text-primary);">
								{user.role}
							</div>
						</div>
					</div>
				</div>
			</div>
		</div>

		<!-- Three Equal Draggable Boxes in Horizontal Row -->
		<div class="grid grid-cols-1 gap-6 transition-opacity duration-200 lg:grid-cols-3" class:opacity-0={!mounted} class:opacity-100={mounted}>
			{#each boxes as boxId, index (boxId)}
				<div
					role="button"
					tabindex="0"
					draggable="true"
					ondragstart={() => handleDragStart(boxId)}
					ondragend={handleDragEnd}
					ondragover={(e) => handleDragOver(e, index)}
					ondrop={(e) => handleDrop(e, index)}
					class="cursor-move transition-opacity duration-200"
					class:opacity-50={draggedBox === boxId}
					class:ring-2={dragOverIndex === index && draggedBox !== boxId}
					class:ring-teal-500={dragOverIndex === index && draggedBox !== boxId}
					style="min-height: 500px;"
				>
					{#if boxId === 'logs'}
						<!-- Today's Logs Box -->
						<div class="flex w-full flex-col border-2" style="border-color: var(--border-primary);">
							<div
								class="border-b-2 px-6 py-4"
								style="border-color: var(--border-primary); background-color: var(--bg-primary);"
							>
								<h2 class="text-xl font-bold" style="color: var(--text-primary);">Today's Logs</h2>
							</div>
							<div class="flex-1 overflow-auto px-6 py-6" style="background-color: var(--bg-primary);">
								{#if todaysLogs.length === 0}
									<div style="color: var(--text-secondary);">No logs due today</div>
								{:else}
									<ul class="space-y-2">
										{#each todaysLogs as log}
											<li style="color: var(--text-primary); overflow: hidden;">
												- <span
													class="inline-block max-w-full overflow-hidden text-ellipsis whitespace-nowrap"
													title={log.template_name}>{log.template_name}</span
												>
												{log.period ? `(${log.period})` : ''}
											</li>
										{/each}
									</ul>
								{/if}
							</div>
						</div>
					{:else if boxId === 'clock'}
						<!-- Clock In/Out Box -->
						<ClockInOut initialStatus={clockStatus} />
					{:else if boxId === 'actions'}
						<!-- Quick Actions Box -->
						<div class="flex w-full flex-col border-2" style="border-color: var(--border-primary);">
							<div
								class="border-b-2 px-6 py-4"
								style="border-color: var(--border-primary); background-color: var(--bg-primary);"
							>
								<h2 class="text-xl font-bold" style="color: var(--text-primary);">Quick Actions</h2>
							</div>
							<div class="flex flex-1 flex-col px-6 py-6" style="background-color: var(--bg-primary);">
								<div class="flex flex-col gap-3">
									<button
										onclick={handleCreateNewTemplate}
										class="cursor-pointer border-2 px-6 py-2 font-medium transition-opacity hover:opacity-80"
										style="border-color: var(--border-primary); color: var(--text-primary); background-color: var(--bg-primary);"
									>
										+ Create New Template
									</button>
									<button
										onclick={handleViewReports}
										class="cursor-pointer border-2 px-6 py-2 font-medium transition-opacity hover:opacity-80"
										style="border-color: var(--border-primary); color: var(--text-primary); background-color: var(--bg-primary);"
									>
										View Reports
									</button>
								</div>
							</div>
						</div>
					{/if}
				</div>
			{/each}
		</div>
	</div>
</div>

<style>
	[draggable='true'] {
		user-select: none;
		-webkit-user-select: none;
		-moz-user-select: none;
		-ms-user-select: none;
	}
</style>
