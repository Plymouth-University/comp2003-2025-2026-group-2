<script lang="ts">
	import { api } from '$lib/api';
	import type { components } from '$lib/api-types';
	import type { PageData } from './$types';
	import InviteModal from './InviteModal.svelte';
	import InviteRow from './InviteRow.svelte';
	import SideBar from './SideBar.svelte';
	import UserRow from './UserRow.svelte';

	export type Member = components['schemas']['GetCompanyMembersResponse']['members'][0];
	export type Invitation = components['schemas']['GetPendingInvitationsResponse']['invitations'][0];

	const data = $props<{ data: PageData }>();
	const members = $state(data.data.members);
	const invitations = $state(data.data.invitations);
	const user = $state(data.data.user);

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

	const cancelInvitation = async (invitationId: string) => {
		const invitation = invitations.find((inv: Invitation) => inv.id === invitationId);
		if (!invitation) return;

		if (!confirm(`Cancel invitation for ${invitation.email}?`)) {
			return;
		}

		try {
			const response = await api.PUT('/auth/invitations/cancel', {
				body: { invitation_id: invitationId }
			});

			if (response.error) {
				alert(`Failed to cancel invitation: ${response.error || 'Unknown error'}`);
				return;
			}

			invitations.splice(
				invitations.findIndex((inv: Invitation) => inv.id === invitationId),
				1
			);
		} catch (error) {
			alert(`Error cancelling invitation: ${error}`);
		}
	};

	const removeMember = async (email: string) => {
		const member = members.find((m: Member) => m.email === email);
		if (!member) return;
		if (member.email === user.email) {
			alert("You cannot remove yourself.");
			return;
		}

		if (!confirm(`Remove ${member.first_name} ${member.last_name} (${email})?`)) {
			return;
		}

		try {
			const response = await api.DELETE('/auth/admin/remove-member', {
				body: { email }
			});

			if (response.error) {
				alert(`Failed to remove member: ${response.data || 'Unknown error'}`);
				return;
			}

			members.splice(
				members.findIndex((m: Member) => m.email === email),
				1
			);
			if (selectedUser?.email === email) {
				selectedUser = null;
			}
		} catch (error) {
			alert(`Error removing member: ${error}`);
		}
	};
</script>

<svelte:head>
	<title>Users Administration</title>
</svelte:head>
<main class="flex min-h-full bg-bg-secondary">
	<div class="flex grow overflow-hidden">
		<div class="mx-auto md:w-full">
			<div class="flex-1 gap-1 overflow-auto p-6">
				<div id="eventHide" class="flex flex-auto flex-col">
					{#each invitations as invite (invite.email)}
						<InviteRow {invite} onCancel={cancelInvitation} />
					{/each}
					{#each members as item (item.email)}
						<UserRow {item} {setSelectedUser} onRemove={removeMember} />
					{/each}
					<div class="add-button-container mr-5 flex flex-col place-items-end self-end text-4xl">
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

<style>
	.add-button-container:hover {
		animation: bounce-once 0.6s ease-in-out;
	}

	@keyframes bounce-once {
		0%,
		100% {
			transform: translateY(0);
		}
		50% {
			transform: translateY(-10px);
		}
	}
</style>
