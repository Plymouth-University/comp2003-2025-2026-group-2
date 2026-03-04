<script lang="ts">
	import { api } from '$lib/api';
	import type { Member } from './+page.svelte';
	import ProfilePictureUploader from '$lib/components/ProfilePictureUploader.svelte';
	import { invalidateAll } from '$app/navigation';

	const { selectedUser, loggedInUserRole, updateMember, branches, isReadonlyHQ } = $props<{
		selectedUser: Member | null;
		loggedInUserRole: string;
		updateMember: (
			email: string,
			updates: { first_name: string; last_name: string; role: string; branch_id: string | null }
		) => void;
		branches: Array<{ id: string; name: string }>;
		isReadonlyHQ: boolean;
	}>();

	let firstName = $state('');
	let lastName = $state('');
	let role = $state('');
	let branchId = $state(null as string | null);
	let profilePictureId = $state<string | null>(null);

	let profilePictureUrl = $derived(
		selectedUser?.profile_picture_url
			? selectedUser.profile_picture_url
			: selectedUser?.oauth_picture || null
	);

	$effect(() => {
		if (selectedUser) {
			firstName = selectedUser.first_name;
			lastName = selectedUser.last_name;
			role = selectedUser.role;
			branchId = selectedUser.branch_id || null;
			profilePictureId = selectedUser.profile_picture_id || null;
		}
	});

	async function handlePictureUpload(pictureUrl: string) {
		if (!selectedUser || isReadonlyHQ) return;

		const response = await api.PUT('/auth/admin/update-member', {
			body: {
				email: selectedUser.email,
				first_name: firstName,
				last_name: lastName,
				role: role,
				branch_id: branchId || undefined,
				profile_picture_id: pictureUrl.split('/').pop()
			}
		});

		if (!response.error) {
			updateMember(selectedUser.email, {
				first_name: firstName,
				last_name: lastName,
				role: role,
				branch_id: branchId
			});
			await invalidateAll();
		}
	}

	async function handlePictureDelete() {
		if (!selectedUser || isReadonlyHQ) return;

		const response = await api.PUT('/auth/admin/update-member', {
			body: {
				email: selectedUser.email,
				first_name: firstName,
				last_name: lastName,
				role: role,
				branch_id: branchId || undefined,
				profile_picture_id: null
			}
		});

		if (!response.error) {
			updateMember(selectedUser.email, {
				first_name: firstName,
				last_name: lastName,
				role: role,
				branch_id: branchId
			});
			await invalidateAll();
		}
	}
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
			<div class="mb-4">
				<ProfilePictureUploader
					currentPictureUrl={profilePictureUrl}
					{firstName}
					{lastName}
					disabled={isReadonlyHQ}
					triggerOnImageClick={true}
					showUploadButton={false}
					onUploadComplete={handlePictureUpload}
					onDeleteComplete={handlePictureDelete}
				/>
			</div>
			<input
				class="mb-2 border-2 border-border-primary bg-bg-primary px-3 py-1 text-text-primary"
				id="fname"
				type="text"
				bind:value={firstName}
				required
				placeholder="First Name"
				disabled={isReadonlyHQ}
			/>
			<input
				class="mb-2 border-2 border-border-primary bg-bg-primary px-3 py-1 text-text-primary"
				id="lname"
				type="text"
				bind:value={lastName}
				required
				placeholder="Last Name"
				disabled={isReadonlyHQ}
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
					class="mb-2 rounded border-2 border-border-primary bg-bg-primary px-4 py-2 font-bold text-text-primary hover:opacity-80 disabled:cursor-not-allowed disabled:opacity-50"
					type="button"
					disabled={isReadonlyHQ}
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
				disabled={isReadonlyHQ}
			>
				<option value="staff">Staff</option>
				<option value="branch_manager" disabled={loggedInUserRole == 'branch_manager'}
					>Branch Manager</option
				>
				<option value="company_manager" disabled={loggedInUserRole == 'branch_manager'}
					>Company Manager</option
				>
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
				disabled={isReadonlyHQ}
			>
				<option
					value={null}
					disabled={loggedInUserRole == 'branch_manager' || role == 'branch_manager'}
					>No Branch (HQ)</option
				>
				{#each branches as branch (branch.id)}
					<option value={branch.id}>{branch.name}</option>
				{/each}
			</select>
			<button
				class="m-5 mb-0 rounded border-2 border-border-primary bg-bg-primary px-4 py-2 font-bold text-text-primary hover:opacity-80 disabled:cursor-not-allowed disabled:opacity-50"
				type="button"
				disabled={isReadonlyHQ}
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
