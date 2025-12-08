<script lang="ts">
	const { item, setSelectedUser } = $props<{
		item: {
			email: string;
			first_name: string;
			last_name: string;
			role: string;
		};
		setSelectedUser: (email: string) => void;
	}>();
</script>

<button
	onclick={() => setSelectedUser(item.email)}
	class="mb-3 flex flex-col gap-3 rounded-lg border-2 p-4 transition-all hover:opacity-80 md:flex-row md:items-center md:justify-between"
	style="background-color: var(--bg-primary); border-color: var(--border-primary);"
>
	<div class="flex flex-1 flex-col gap-2">
		<div class="font-semibold" style="color: var(--text-primary);">
			{item.first_name}
			{item.last_name}
		</div>
		<div class="text-sm" style="color: var(--text-secondary);">
			{item.email}
		</div>
	</div>

	<div class="flex items-center gap-2">
		<div
			class="rounded-full px-3 py-1 text-sm font-medium"
			style="background-color: {item.role === 'admin'
				? '#94C5CC'
				: 'var(--bg-secondary)'}; color: {item.role === 'admin'
				? '#000'
				: 'var(--text-secondary)'};"
		>
			{item.role === 'admin' ? 'Admin' : 'Member'}
		</div>
	</div>

	<div class="flex gap-2">
		<a
			href="#remove-user"
			class="rounded px-3 py-1 text-sm font-medium transition-colors hover:underline"
			style="color: var(--text-secondary);"
		>
			Remove
		</a>
	</div>
</button>

<style>
</style>
