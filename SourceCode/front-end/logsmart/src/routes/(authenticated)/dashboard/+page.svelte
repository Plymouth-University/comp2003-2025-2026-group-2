<script lang="ts">
	import type { PageData } from './$types';

	let { data } = $props<{ data: PageData }>();

	// Sample data for today's logs
	const todaysLogs = $state([
		'Kitchen Daily Log (27th)',
		'Kitchen Cleaning Log (w/e 27th)',
		'Weekly Log'
	]);

	// Get user data from server load
	const user = $derived(() => {
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

		return {
			name: fullName || 'User',
			email: data.user.email || '',
			company: data.user.company_name || 'N/A',
			role: data.user.role || 'User',
			initials: initials || '?'
		};
	});
</script>

<div class="h-full w-full" style="background-color: #F8F8F8;">
	<!-- Main Content -->
	<div class="mx-auto max-w-7xl px-6 py-8">
		<!-- Header with User Profile -->
		<div class="mb-8 flex items-start justify-between">
			<h1 class="mb-8 text-3xl font-bold" style="color: #000100;">Dashboard Overview</h1>

			<!-- User Profile Section -->
			<div class="inline-block border-2" style="border-color: #000100;">
				<div class="bg-white px-6 py-4">
					<div class="flex items-center gap-4">
						<!-- Profile Picture (Initials) -->
						<div
							class="flex h-16 w-16 items-center justify-center rounded-full text-xl font-bold text-white"
							style="background-color: #94C5CC;"
						>
							{user().initials}
						</div>
						<!-- User Info -->
						<div class="text-left">
							<div class="font-bold" style="color: #000100;">{user().name}</div>
							<div class="text-sm" style="color: #A1A6B4;">{user().email}</div>
							<div class="text-sm" style="color: #A1A6B4;">{user().company}</div>
							<div class="text-sm font-medium" style="color: #000100;">{user().role}</div>
						</div>
					</div>
				</div>
			</div>
		</div>

		<!-- Today's Logs Section -->
		<div class="mb-8">
			<div class="inline-block border-2" style="border-color: #000100;">
				<div class="border-b-2 bg-white px-6 py-4" style="border-color: #000100;">
					<h2 class="text-xl font-bold" style="color: #000100;">Today's Logs</h2>
				</div>
				<div class="min-h-[280px] min-w-[380px] bg-white px-6 py-6">
					<ul class="space-y-2">
						{#each todaysLogs as log}
							<li style="color: #000100;">- {log}</li>
						{/each}
					</ul>
				</div>
			</div>
		</div>

		<!-- Quick Actions Section -->
		<div>
			<div class="inline-block border-2" style="border-color: #000100;">
				<div class="border-b-2 bg-white px-6 py-4" style="border-color: #000100;">
					<h2 class="text-xl font-bold" style="color: #000100;">Quick Actions</h2>
				</div>
				<div class="min-w-[380px] bg-white px-6 py-6">
					<div class="flex flex-col items-start gap-4">
						<button
							class="border-2 bg-white px-6 py-2 font-medium hover:opacity-80"
							style="border-color: #000100; color: #000100;"
						>
							+ Create New Log
						</button>
						<button
							class="border-2 bg-white px-6 py-2 font-medium hover:opacity-80"
							style="border-color: #000100; color: #000100;"
						>
							View Reports
						</button>
					</div>
				</div>
			</div>
		</div>
	</div>
</div>
