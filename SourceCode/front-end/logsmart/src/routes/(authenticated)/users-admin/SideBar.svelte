<script lang="ts">
	import { api } from '$lib/api';
	import type { Member } from './+page.svelte';
	import PlaceHolderImage from '$lib/assets/placeholder.png';

	const { setSelectedUser, selectedUser, loggedInUserRole, updateMember, branches } = $props<{
		setSelectedUser: (email: string | null) => void;
		selectedUser: Member | null;
		loggedInUserRole: string;
		updateMember: (
			email: string,
			updates: { first_name: string; last_name: string; role: string; branch_id: string | null }
		) => void;
		branches: any[];
	}>();

	let firstName = $state('');
	let lastName = $state('');
	let role = $state('');
	let branchId = $state(null as string | null);

	$effect(() => {
		if (selectedUser) {
			firstName = selectedUser.first_name;
			lastName = selectedUser.last_name;
			role = selectedUser.role;
			branchId = selectedUser.branch_id || null;
		}
	});
</script>

<div
	id="userSidebar"
	class="w-70 border-l-2 border-border-primary bg-bg-primary text-center {selectedUser
		? 'flex'
		: 'hidden'} flex-col items-center p-6"
>
	<span class="mb-1 text-xl font-bold text-text-primary">Profile</span>
	<div class="flex flex-col justify-items-center">
		<form
			class="mb-4 flex flex-col items-center px-8 pt-2 pb-8"
			onsubmit={(e) => e.preventDefault()}
		>
			<img class="h-50 w-50" src={PlaceHolderImage} alt="User Profile" />
			<input
				class="mb-2 border-2 border-border-primary bg-bg-primary px-3 py-1 text-text-primary"
				id="fname"
				type="text"
				bind:value={firstName}
				required
				placeholder="First Name"
			/>
			<input
				class="mb-2 border-2 border-border-primary bg-bg-primary px-3 py-1 text-text-primary"
				id="lname"
				type="text"
				bind:value={lastName}
				required
				placeholder="Last Name"
			/>
			<input
				class="mb-2 border-2 border-border-primary bg-bg-primary px-3 py-1 text-text-primary"
				id="sidebar-email"
				type="text"
				value={selectedUser?.email}
				disabled
				required
				placeholder="Email"
			/>
			<div class="flex flex-col gap-4 md:flex-row">
				<input
					class="mb-2 w-[60%] cursor-not-allowed border-2 border-border-secondary bg-bg-secondary px-3 py-1 text-text-secondary select-none"
					id="password"
					type="text"
					value="*******"
					disabled
				/>
				<button
					class="mb-2 cursor-pointer rounded border-2 border-border-primary bg-bg-primary px-4 py-2 font-bold text-text-primary hover:opacity-80"
					type="button"
					onclick={() => {
						if (selectedUser)
							api.POST('/auth/password/request-reset', { body: { email: selectedUser?.email } });
					}}>Reset</button
				>
			</div>
			<label for="sidebar-role" class="mb-1 text-sm font-medium text-text-primary">Role</label>
			<select
				class="mb-3 border-2 border-border-primary bg-bg-primary px-3 py-1 text-text-primary"
				name="role"
				id="sidebar-role"
				bind:value={role}
			>
				<option value="staff">Staff</option>
				<option value="branch_manager">Branch Manager</option>
				<option value="company_manager" disabled={loggedInUserRole == "branch_manager"}>Company Manager</option>
				{#if loggedInUserRole === 'logsmart_admin'}
					<option id="logsmart_adminRole" value="logsmart_admin">Internal Admin</option>
				{/if}
			</select>
			<label for="sidebar-branch" class="mb-1 text-sm font-medium text-text-primary">Branch</label>
			<select
				class="mb-3 border-2 border-border-primary bg-bg-primary px-3 py-1 text-text-primary"
				name="branch"
				id="sidebar-branch"
				bind:value={branchId}
			>
				<option value={null}>No Branch (HQ)</option>
				{#each branches as branch}
					<option value={branch.id}>{branch.name}</option>
				{/each}
			</select>
			<button
				class="m-5 mb-0 cursor-pointer rounded border-2 border-border-primary bg-bg-primary px-4 py-2 font-bold text-text-primary hover:opacity-80"
				type="button"
				onclick={async () => {
					const response = await api.PUT('/auth/admin/update-member', {
						body: {
							email: selectedUser?.email as string,
							first_name: firstName,
							last_name: lastName,
							role: role,
							branch_id: branchId || undefined
						}
					});

					if (!response.error && selectedUser) {
						updateMember(selectedUser.email, {
							first_name: firstName,
							last_name: lastName,
							role: role,
							branch_id: branchId
						});
					} else if (response.error) {
						alert(`Failed to update member: ${response.error.error}`);
					}
				}}>Save</button
			>
		</form>
	</div>
</div>
