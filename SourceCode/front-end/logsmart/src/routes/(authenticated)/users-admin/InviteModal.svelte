<script lang="ts">
	import { api } from '$lib/api';

	const { showingCreateModel, setShowingCreateModel } = $props<{
		showingCreateModel: boolean;
		setShowingCreateModel: (show: boolean) => void;
	}>();
	let email = $state('');
</script>

<div
	id="registration-modal"
	tabindex="-1"
	aria-hidden={!showingCreateModel}
	class="fixed top-0 right-0 left-0 z-50 {showingCreateModel
		? 'flex'
		: 'hidden'} h-[calc(100%-1rem)] max-h-full w-full items-center justify-center overflow-x-hidden overflow-y-auto md:inset-0"
>
	<div class="relative max-h-full w-full max-w-md p-4">
		<div
			class="rounded-base relative border-2 p-4 shadow-sm md:p-6"
			style="background-color: var(--bg-primary); border-color: var(--border-primary);"
		>
			<div class="flex items-center justify-between border-b-2 pb-4 md:pb-5" style="border-color: var(--border-primary);">
				<h3 class="text-lg font-medium" style="color: var(--text-primary);">REGISTER NEW USER</h3>
				<button
					type="button"
					class="rounded-base ms-auto inline-flex h-9 w-9 cursor-pointer items-center justify-center bg-transparent text-sm hover:opacity-80"
					style="color: var(--text-secondary);"
					onclick={() => setShowingCreateModel(false)}
				>
					<svg
						class="h-5 w-5"
						aria-hidden="true"
						xmlns="http://www.w3.org/2000/svg"
						width="24"
						height="24"
						fill="none"
						viewBox="0 0 24 24"
						><path
							stroke="currentColor"
							stroke-linecap="round"
							stroke-linejoin="round"
							stroke-width="2"
							d="M6 18 17.94 6M18 18 6.06 6"
						/></svg
					>
					<span class="sr-only">Close modal</span>
				</button>
			</div>
			<form action="#" class="flex flex-col pt-4 md:pt-6">
				<div class="mb-4">
					<label for="email" class="mb-2.5 block text-sm font-medium" style="color: var(--text-primary);"
						>New user's email</label
					>
					<input
						type="email"
						id="email"
						bind:value={email}
						class="rounded-base block w-full border-2 px-3 py-2.5 text-sm shadow-xs focus:outline-none focus:ring-2"
						style="background-color: var(--bg-primary); border-color: var(--border-primary); color: var(--text-primary);"
						placeholder="example@company.com"
						required
					/>
				</div>
				<button
					class="flex cursor-pointer self-center rounded border-2 px-5 py-2.5 text-center text-sm font-medium hover:opacity-80"
					style="border-color: var(--border-primary); background-color: var(--bg-primary); color: var(--text-primary);"
					aria-label="Send email"
					onclick={async () => {
						let { error } = await api.POST('/auth/invitations/send', { body: { email: email } });
						if (!error) {
							setShowingCreateModel(false);
						} else {
							console.error('Error sending invite:', error);
						}
					}}
				>
					<span class="text-sm font-semibold" style="color: var(--text-primary);">Send Invite</span>
				</button>
			</form>
		</div>
	</div>
</div>
