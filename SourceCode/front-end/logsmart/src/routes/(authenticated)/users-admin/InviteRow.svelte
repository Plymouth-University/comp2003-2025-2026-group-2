<script lang="ts">
	const { invite, onCancel } = $props<{
		invite: {
			email: string;
			id: string;
			expires_at: string;
		};
		onCancel: (invitationId: string) => Promise<void>;
	}>();
</script>

<div
	class="mb-3 flex flex-col gap-3 rounded-lg border-2 border-dashed p-4 transition-all hover:opacity-80 md:flex-row md:items-center md:justify-between"
	style="background-color: var(--orange-light); border-color: var(--orange);"
>
	<div class="flex flex-1 flex-col gap-2">
		<div class="flex items-center gap-2">
			<div class="font-semibold" style="color: grey;">
				{invite.email}
			</div>
			<span class="text-xs font-medium" style="color: var(--orange-dark);">⏳ Pending</span>
		</div>
		<div class="self-start text-sm" style="color: var(--text-secondary);">
			Expires: {new Date(invite.expires_at).toLocaleDateString()}
		</div>
	</div>

	<div class="flex items-center gap-2">
		<div
			class="rounded-full border px-3 py-1 text-sm font-medium"
			style="background-color: var(--orange); border-color: var(--orange); color: var(--border-primary);"
		>
			Pending Invitation
		</div>
	</div>

	<div class="flex gap-2">
		<button
			type="button"
			onclick={() => onCancel(invite.id)}
			class="rounded px-3 py-1 text-sm font-medium transition-colors hover:underline"
			style="color: var(--text-secondary);"
		>
			Cancel
		</button>
	</div>
</div>

<style>
</style>
