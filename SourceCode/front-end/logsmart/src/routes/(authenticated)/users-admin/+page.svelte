<script lang="ts">
	import type { components } from '$lib/api-types';
	import type { PageData } from './$types';
	import InviteModal from './InviteModal.svelte';
	import SideBar from './SideBar.svelte';
	import UserRow from './UserRow.svelte';

	export type Member = components['schemas']['GetCompanyMembersResponse']['members'][0];

	const data = $props<{ data: PageData }>();
	const members = $state(data.data.members);

	let showingCreateModel = $state(false);

	let selectedUser = $state(null as Member | null);

	const setSelectedUser = (email: string | null) => {
		if (email === selectedUser?.email) {
			selectedUser = null;
			return;
		}
		selectedUser = members.find((member: Member) => member.email === email) || null;
	};
	const setShowingCreateModel = (show: boolean) => {
		showingCreateModel = show;
	};
</script>

<svelte:head>
	<title>Users Administration</title>
</svelte:head>
<main class="min-h-full bg-bg-secondary">
	<div class="overflow-hidden grow-1 flex">
		<div class="mx-auto md:w-full">
			<div class="flex-1 gap-1 overflow-auto p-6">
				<div id="eventHide" class="flex flex-auto flex-col">
					{#each members as item (item.email)}
						<UserRow {item} {setSelectedUser} />
					{/each}
					<div class="mr-5 flex flex-col place-items-end self-end text-4xl hover:animate-bounce">
						<button
							class="z-80 h-20 w-20 cursor-pointer self-end rounded-full border-4 border-border-primary bg-bg-primary text-text-primary drop-shadow-lg duration-300 hover:drop-shadow-2xl"
							type="button"
							onclick={() => (showingCreateModel = !showingCreateModel)}
						>
							<span>&#10133;</span>
						</button>
						<span class="m-3 mt-2 text-sm text-text-primary">Add New</span>
					</div>
					<InviteModal {showingCreateModel} {setShowingCreateModel} />
				</div>
			</div>
		</div>
		<SideBar {selectedUser} {setSelectedUser} />
	</div>
</main>
