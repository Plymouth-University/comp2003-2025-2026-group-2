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
			class="rounded-base border-border-primary bg-bg-primary relative border-2 p-4 shadow-sm md:p-6"
		>
			<div class="border-border-primary flex items-center justify-between border-b-2 pb-4 md:pb-5">
				<h3 class="text-text-primary text-lg font-medium">REGISTER NEW USER</h3>
				<button
					type="button"
					class="rounded-base text-text-secondary ms-auto inline-flex h-9 w-9 cursor-pointer items-center justify-center bg-transparent text-sm hover:opacity-80"
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
					<label for="email" class="text-text-primary mb-2.5 block text-sm font-medium"
						>New user's email</label
					>
					<input
						type="email"
						id="email"
						bind:value={email}
						class="rounded-base border-border-primary bg-bg-primary text-text-primary block w-full border-2 px-3 py-2.5 text-sm shadow-xs focus:ring-2 focus:outline-none"
						placeholder="example@company.com"
						required
					/>
				</div>
				<button
					class="border-border-primary bg-bg-primary text-text-primary flex cursor-pointer self-center rounded border-2 px-5 py-2.5 text-center text-sm font-medium hover:opacity-80"
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
					<span class="text-text-primary text-sm font-semibold">Send Invite</span>
				</button>
			</form>
		</div>
	</div>
</div>
