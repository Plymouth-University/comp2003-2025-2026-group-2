<script lang="ts">
	import { api } from '$lib/api';
	import type { Member } from './+page.svelte';
	import PictureUploader from '$lib/components/PictureUploader.svelte';
	import { invalidateAll } from '$app/navigation';

	const { selectedUser, loggedInUserRole, updateMember, branches, isReadonlyHQ, onClose } = $props<{
		selectedUser: Member | null;
		loggedInUserRole: string;
		updateMember: (
			email: string,
			updates: { first_name: string; last_name: string; role: string; branch_id: string | null }
		) => void;
		branches: Array<{ id: string; name: string }>;
		isReadonlyHQ: boolean;
		onClose?: () => void;
	}>();

	let windowWidth = $state(1920);
	let isMobile = $derived(windowWidth < 740);

	if (typeof window !== 'undefined') {
		windowWidth = window.innerWidth;
	}

	function handleResize() {
		windowWidth = window.innerWidth;
	}
	let firstName = $state('');
	let lastName = $state('');
	let role = $state('');
	let branchId = $state(null as string | null);

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
		}
	});

	async function handlePictureUpload(pictureUrl: string) {
		if (!selectedUser || isReadonlyHQ) return;
		updateMember(selectedUser.email, {
			first_name: firstName,
			last_name: lastName,
			role: role,
			branch_id: branchId,
			profile_picture_url: pictureUrl,
			profile_picture_id: pictureUrl.split('/').pop() || null
		});
		await invalidateAll();
	}

	async function handlePictureDelete() {
		if (!selectedUser || isReadonlyHQ) return;
		updateMember(selectedUser.email, {
			first_name: firstName,
			last_name: lastName,
			role: role,
			branch_id: branchId,
			profile_picture_url: null,
			profile_picture_id: null
		});
		await invalidateAll();
	}
</script>

<svelte:window onresize={handleResize} />

{#if isMobile}
	<div
		id="userSidebarModal"
		tabindex="-1"
		aria-hidden={!selectedUser}
		class="fixed top-0 right-0 left-0 z-50 {selectedUser
			? 'flex'
			: 'hidden'} h-[calc(100%-1rem)] max-h-full w-full items-center justify-center overflow-x-hidden overflow-y-auto md:inset-0"
	>
		<div class="relative max-h-full w-full max-w-md p-4">
			<div
				class="rounded-base relative border-2 border-border-primary bg-bg-primary p-4 shadow-sm md:p-6"
			>
				<div
					class="flex items-center justify-between border-b-2 border-border-primary pb-4 md:pb-5"
				>
					<span class="text-lg font-medium text-text-primary">Profile</span>
					<button
						type="button"
						class="rounded-base ms-auto inline-flex h-9 w-9 cursor-pointer items-center justify-center bg-transparent text-sm text-text-secondary hover:opacity-80"
						onclick={() => onClose?.()}
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
				<div class="flex flex-col justify-items-center py-4">
					<form
						class="mb-4 flex flex-col items-center px-2 pt-2 pb-4"
						onsubmit={(e) => e.preventDefault()}
					>
						<div class="mb-4">
							<PictureUploader
								type="pfp"
								currentPictureUrl={profilePictureUrl}
								{firstName}
								{lastName}
								disabled={isReadonlyHQ}
								triggerOnImageClick={true}
								showUploadButton={false}
								targetUserEmail={selectedUser?.email || ''}
								onUploadComplete={handlePictureUpload}
								onDeleteComplete={handlePictureDelete}
							/>
						</div>
						<input
							class="mb-2 w-full border-2 border-border-primary bg-bg-primary px-3 py-1 text-text-primary"
							id="fname"
							type="text"
							bind:value={firstName}
							required
							placeholder="First Name"
							disabled={isReadonlyHQ}
						/>
						<input
							class="mb-2 w-full border-2 border-border-primary bg-bg-primary px-3 py-1 text-text-primary"
							id="lname"
							type="text"
							bind:value={lastName}
							required
							placeholder="Last Name"
							disabled={isReadonlyHQ}
						/>
						<input
							class="mb-2 w-full border-2 border-border-primary bg-bg-primary px-3 py-1 text-text-primary"
							id="sidebar-email"
							type="text"
							value={selectedUser?.email}
							disabled
							required
							placeholder="Email"
						/>
						<div class="flex w-full flex-row gap-2 sm:flex-row">
							<input
								class="mb-2 grow cursor-not-allowed border-2 border-border-secondary bg-bg-secondary px-3 py-1 text-text-secondary select-none"
								id="password"
								type="text"
								value="*******"
								disabled
							/>
							<button
								class="mb-2 ml-auto rounded border-2 border-border-primary bg-bg-primary px-2 py-2 font-bold text-text-primary hover:opacity-80 disabled:cursor-not-allowed disabled:opacity-50 sm:px-4"
								type="button"
								disabled={isReadonlyHQ}
								onclick={() => {
									if (selectedUser)
										api.POST('/auth/password/request-reset', {
											body: { email: selectedUser?.email }
										});
								}}>Reset</button
							>
						</div>
						<label for="sidebar-role-mobile" class="mb-1 text-sm font-medium text-text-primary"
							>Role</label
						>
						<select
							class="mb-3 w-full border-2 border-border-primary bg-bg-primary px-3 py-1 text-text-primary"
							name="role"
							id="sidebar-role-mobile"
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
						<label for="sidebar-branch-mobile" class="mb-1 text-sm font-medium text-text-primary"
							>Branch</label
						>
						<select
							class="mb-3 w-full border-2 border-border-primary bg-bg-primary px-3 py-1 text-text-primary"
							name="branch"
							id="sidebar-branch-mobile"
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
		</div>
	</div>
{:else}
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
					<PictureUploader
						type="pfp"
						currentPictureUrl={profilePictureUrl}
						{firstName}
						{lastName}
						disabled={isReadonlyHQ}
						triggerOnImageClick={true}
						showUploadButton={false}
						targetUserEmail={selectedUser?.email || ''}
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
				<label for="sidebar-branch" class="mb-1 text-sm font-medium text-text-primary">Branch</label
				>
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
{/if}
