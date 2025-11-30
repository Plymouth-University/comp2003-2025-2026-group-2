<script lang="ts">
	import { api } from "$lib/api";

	const { showingCreateModel, setShowingCreateModel } = $props<{ showingCreateModel: boolean; setShowingCreateModel: (show: boolean) => void }>();
    let email = $state("");
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
			class="bg-neutral-primary-soft border-default rounded-base relative border p-4 shadow-sm md:p-6"
		>
			<div class="border-default flex items-center justify-between border-b pb-4 md:pb-5">
				<h3 class="text-heading text-lg font-medium">REGISTER NEW USER</h3>
				<button
					type="button"
					class="text-body hover:bg-neutral-tertiary hover:text-heading rounded-base ms-auto inline-flex h-9 w-9 items-center justify-center bg-transparent text-sm"
					onclick={() => (setShowingCreateModel(false))}
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
			<form action="#" class="pt-4 md:pt-6 flex flex-col">
				<div class="mb-4">
					<label for="email" class="text-heading mb-2.5 block text-sm font-medium"
						>New user's email</label
					>
					<input
						type="email"
						id="email"
                        bind:value={email}
						class="bg-neutral-secondary-medium border-default-medium text-heading rounded-base focus:ring-brand focus:border-brand placeholder:text-body block w-full border px-3 py-2.5 text-sm shadow-xs"
						placeholder="example@company.com"
						required
					/>
				</div>
				<button class="flex self-center rounded border px-5 py-2.5 text-center text-sm font-medium hover:bg-brand-dark" aria-label="Send email" onclick={async () => {
                    let {error} = await api.POST('/auth/invitations/send', {body: {email: email}});
                    if (!error) {
                        setShowingCreateModel(false);
                    } else {
                        console.error("Error sending invite:", error);
                    }
                }}>
					<span
                        class="text-black text-sm font-semibold"
                        >Send Invite</span
                    >
				</button>
			</form>
		</div>
	</div>
</div>
