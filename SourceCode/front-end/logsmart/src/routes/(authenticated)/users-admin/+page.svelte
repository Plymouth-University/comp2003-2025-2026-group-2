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
<main class="bg-bg-secondary min-h-full">
	<div class="overflow-none flex">
		<div class="mx-auto md:w-full">
			<div class="flex-1 gap-1 overflow-auto p-6">
				<div id="eventHide" class="flex flex-auto flex-col">
					{#each members as item (item.email)}
						<UserRow {item} {setSelectedUser} />
					{/each}
					<div class="mr-5 flex flex-col place-items-end self-end text-4xl hover:animate-bounce">
						<button
							class="border-border-primary bg-bg-primary text-text-primary z-80 h-20 w-20 cursor-pointer self-end rounded-full border-4 drop-shadow-lg duration-300 hover:drop-shadow-2xl"
							type="button"
							onclick={() => (showingCreateModel = !showingCreateModel)}
						>
							<span>&#10133;</span>
						</button>
						<span class="text-text-primary m-3 mt-2 text-sm">Add New</span>
					</div>
					<InviteModal {showingCreateModel} {setShowingCreateModel} />
				</div>
			</div>
		</div>
		<SideBar {selectedUser} {setSelectedUser} />
	</div>
</main>
