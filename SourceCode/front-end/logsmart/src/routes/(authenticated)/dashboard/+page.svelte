<script lang="ts">
	// Sample data for today's logs
	const todaysLogs = [
		'Kitchen Daily Log (27th)',
		'Kitchen Cleaning Log (w/e 27th)',
		'Weekly Log'
	];

	// Sample user data
	const user = {
		name: 'John Smith',
		email: 'john.smith@example.com',
		company: 'LogSmart Industries',
		role: 'Manager',
		initials: 'JS'
	};
</script>

<div class="min-h-screen" style="background-color: #F8F8F8;">
	<!-- Main Content -->
	<div class="mx-auto max-w-7xl px-6 py-8">
		<!-- Header with User Profile -->
		<div class="flex items-start justify-between mb-8">
			<h1 class="text-3xl font-bold mb-8" style="color: #000100;">Dashboard Overview</h1>
			
			<!-- User Profile Section -->
			<div class="border-2 inline-block" style="border-color: #000100;">
				<div class="bg-white px-6 py-4">
					<div class="flex items-center gap-4">
						<!-- Profile Picture (Initials) -->
						<div class="w-16 h-16 rounded-full flex items-center justify-center text-white text-xl font-bold" style="background-color: #94C5CC;">
							{user.initials}
						</div>
						<!-- User Info -->
						<div class="text-left">
							<div class="font-bold" style="color: #000100;">{user.name}</div>
							<div class="text-sm" style="color: #A1A6B4;">{user.email}</div>
							<div class="text-sm" style="color: #A1A6B4;">{user.company}</div>
							<div class="text-sm font-medium" style="color: #000100;">{user.role}</div>
						</div>
					</div>
				</div>
			</div>
		</div>

		<!-- Today's Logs Section -->
		<div class="mb-8">
			<div class="border-2 inline-block" style="border-color: #000100;">
				<div class="border-b-2 bg-white px-6 py-4" style="border-color: #000100;">
					<h2 class="text-xl font-bold" style="color: #000100;">Today's Logs</h2>
				</div>
				<div class="bg-white px-6 py-6 min-w-[380px] min-h-[280px]">
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
			<div class="border-2 inline-block" style="border-color: #000100;">
				<div class="border-b-2 bg-white px-6 py-4" style="border-color: #000100;">
					<h2 class="text-xl font-bold" style="color: #000100;">Quick Actions</h2>
				</div>
				<div class="bg-white px-6 py-6 min-w-[380px]">
					<div class="flex flex-col gap-4 items-start">
						<button class="border-2 bg-white px-6 py-2 font-medium hover:opacity-80" style="border-color: #000100; color: #000100;">
							+ Create New Log
						</button>
						<button class="border-2 bg-white px-6 py-2 font-medium hover:opacity-80" style="border-color: #000100; color: #000100;">
							View Reports
						</button>
					</div>
				</div>
			</div>
		</div>
	</div>
</div>
