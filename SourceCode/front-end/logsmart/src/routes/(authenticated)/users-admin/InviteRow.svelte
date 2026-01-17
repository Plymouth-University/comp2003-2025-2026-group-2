<script lang="ts">
	const { invite } = $props<{
		invite: {
			email: string;
			id: string;
			expires_at: string;
		};
	}>();
</script>

<button
	class="mb-3 flex flex-col gap-3 rounded-lg border-2 border-dashed p-4 transition-all hover:opacity-80 md:flex-row md:items-center md:justify-between"
	style="background-color: #FFF9E6; border-color: #FFD700;"
>
	<div class="flex flex-1 flex-col gap-2">
		<div class="flex items-center gap-2">
			<div class="font-semibold" style="color: var(--text-primary);">
				{invite.email}
			</div>
			<span class="text-xs font-medium" style="color: #B8860B;">⏳ Pending</span>
		</div>
		<div class="self-start text-sm" style="color: var(--text-secondary);">
			Expires: {new Date(invite.expires_at).toLocaleDateString()}
		</div>
	</div>

	<div class="flex items-center gap-2">
		<div
			class="rounded-full border px-3 py-1 text-sm font-medium"
			style="background-color: #FFE55C; border-color: #FFD700; color: #000;"
		>
			Pending Invitation
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
