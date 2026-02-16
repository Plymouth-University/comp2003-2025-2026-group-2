<script lang="ts">
	import { page } from '$app/state';
	import { api } from '$lib/api';
	import type { components } from '$lib/api-types';

	type Branch = components['schemas']['BranchDto'];

	const { showingCreateModel, setShowingCreateModel, branches } = $props<{
		showingCreateModel: boolean;
		setShowingCreateModel: (show: boolean) => void;
		branches: any[];
	}>();
	let email = $state('');
	let role = $state('staff');
	let branchId = $state(null as string | null);
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
			class="rounded-base relative border-2 border-border-primary bg-bg-primary p-4 shadow-sm md:p-6"
		>
			<div class="flex items-center justify-between border-b-2 border-border-primary pb-4 md:pb-5">
				<h3 class="text-lg font-medium text-text-primary">REGISTER NEW USER</h3>
				<button
					type="button"
					class="rounded-base ms-auto inline-flex h-9 w-9 cursor-pointer items-center justify-center bg-transparent text-sm text-text-secondary hover:opacity-80"
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
					<label for="email" class="mb-2.5 block text-sm font-medium text-text-primary"
						>New user's email</label
					>
					<input
						type="email"
						id="email"
						bind:value={email}
						class="rounded-base block w-full border-2 border-border-primary bg-bg-primary px-3 py-2.5 text-sm text-text-primary shadow-xs focus:ring-2 focus:outline-none"
						placeholder="example@company.com"
						required
					/>
				</div>
				<div class="mb-4">
					<label for="invite-role" class="mb-2.5 block text-sm font-medium text-text-primary"
						>Role</label
					>
					<select
						id="invite-role"
						bind:value={role}
						class="rounded-base block w-full border-2 border-border-primary bg-bg-primary px-3 py-2.5 text-sm text-text-primary shadow-xs focus:ring-2 focus:outline-none"
					>
						<option value="staff">Staff</option>
						<option value="branch_manager">Branch Manager</option>
						<option value="company_manager">Company Manager</option>
					</select>
				</div>
				<div class="mb-6">
					<label for="invite-branch" class="mb-2.5 block text-sm font-medium text-text-primary"
						>Branch (Optional)</label
					>
					<select
						id="invite-branch"
						bind:value={branchId}
						class="rounded-base block w-full border-2 border-border-primary bg-bg-primary px-3 py-2.5 text-sm text-text-primary shadow-xs focus:ring-2 focus:outline-none"
					>
						<option value={null}>Company Wide / Headquarters</option>
						{#each branches as branch}
							<option value={branch.id}>{branch.name}</option>
						{/each}
					</select>
				</div>
				<button
					class="flex cursor-pointer self-center rounded border-2 border-border-primary bg-bg-primary px-5 py-2.5 text-center text-sm font-medium text-text-primary hover:opacity-80"
					disabled={!email || !/^[^\s@]+@[^\s@]+\.[^\s@]+$/.test(email)}
					aria-label="Send Invite"
					onclick={async () => {
						let { error } = await api.POST('/auth/invitations/send', {
							body: {
								email: email,
								role: role as any,
								branch_id: branchId || undefined
							}
						});
						if (!error) {
							setShowingCreateModel(false);
							window.location.reload();
						} else {
							console.error('Error sending invite:', error);
							alert(`Error: ${error.error}`);
						}
					}}
				>
					<span class="text-sm font-semibold text-text-primary">Send Invite</span>
				</button>
			</form>
		</div>
	</div>
</div>
