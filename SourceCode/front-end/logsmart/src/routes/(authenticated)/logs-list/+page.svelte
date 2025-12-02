<script lang="ts">
	let logsTemplates = $state([
		{ id: 1, date: '10/10/2025', label: 'FREEZER TEMPERATURES', frequency: 'Weekly' },
		{ id: 2, date: '10/10/2025', label: 'DISHWASHER TEMPERATURES', frequency: 'Weekly' },
		{ id: 3, date: '10/10/2025', label: 'DISHWASHER CLEANLINESS', frequency: 'Daily' },
		{ id: 4, date: '10/10/2025', label: 'FOOD DATE CODES CHECKED', frequency: 'Daily' },
		{ id: 5, date: '10/10/2025', label: 'MIXER CLEANLINESS', frequency: 'Daily' }
	]);
</script>

<svelte:head>
	<title>Logs List</title>
</svelte:head>
<main>
	<div class="min-h-screen" style="background-color: var(--bg-secondary);">
		<div class="flex-1 gap-8 overflow-auto">
			<div class="mx-auto max-w-screen">
				{#key logsTemplates}
					{#each logsTemplates as item}
						<div
							class="overflow-x-none flex h-30 flex-auto flex-row content-between border-t-2 border-b-2 text-2xl"
							style="margin-top: -2px; display:flex; background-color: var(--bg-primary); border-color: var(--border-primary); color: var(--text-primary);"
						>
							<div class="userBox-rows"><b>{item.date}</b></div>
							<div class="userBox-rows">{item.label}</div>
							<div class="userBox-rows">-</div>
							<div class="userBox-rows" style="flex:1">{item.frequency}</div>
							<button class="userBox-rows" style="cursor:pointer; font-size:35px;">&#128393;</button
							>
							<button class="userBox-rows" style="cursor:pointer; font-size:35px;">&#128438;</button
							>
						</div>
					{/each}
				{/key}
			</div>
		</div>
	</div>
</main>

<style>
	.userBox-rows {
		padding: 3%;
	}
</style>
