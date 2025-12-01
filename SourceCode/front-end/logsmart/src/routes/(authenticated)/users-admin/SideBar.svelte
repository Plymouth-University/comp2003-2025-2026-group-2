<script lang="ts">
	import { api } from '$lib/api';
	import type { Member } from './+page.svelte';
	import PlaceHolderImage from '$lib/assets/placeholder.png';

	const { setSelectedUser, selectedUser } = $props<{
		setSelectedUser: (email: string | null) => void;
		selectedUser: Member | null;
	}>();
</script>

<div
	id="userSidebar"
	class="w-70 border-l-2 text-center {selectedUser
		? 'flex'
		: 'hidden'} flex-col items-center p-6"
	style="border-color: var(--border-primary); background-color: var(--bg-primary);"
>
	<span class="mb-1 text-xl font-bold" style="color: var(--text-primary);">Profile</span>
	<div class="flex flex-col justify-items-center">
		<form class="mb-4 flex flex-col items-center px-8 pt-2 pb-8">
			<img class="h-50 w-50" src={PlaceHolderImage} alt="User Profile" />
			<input
				class="mb-2 border-2 px-3 py-1"
				style="border-color: var(--border-primary); background-color: var(--bg-primary); color: var(--text-primary);"
				id="fname"
				type="text"
				value={selectedUser?.first_name}
				required
				placeholder="First Name"
			/>
			<input
				class="mb-2 border-2 px-3 py-1"
				style="border-color: var(--border-primary); background-color: var(--bg-primary); color: var(--text-primary);"
				id="lname"
				type="text"
				value={selectedUser?.last_name}
				required
				placeholder="Last Name"
			/>
			<input
				class="mb-2 border-2 px-3 py-1"
				style="border-color: var(--border-primary); background-color: var(--bg-primary); color: var(--text-primary);"
				id="email"
				type="text"
				value={selectedUser?.email}
				required
				placeholder="Email"
			/>
			<div class="flex flex-col gap-4 md:flex-row">
				<input
					class="mb-2 cursor-not-allowed select-none border-2 px-3 py-1"
					style="width:60%; border-color: var(--border-secondary); background-color: var(--bg-secondary); color: var(--text-secondary); -webkit-user-select: none; user-select: none; -ms-user-select: none;"
					id="password"
					type="text"
					value="*******"
					disabled
				/>
				<button
					class="mb-2 cursor-pointer rounded border-2 px-4 py-2 font-bold hover:opacity-80"
					style="border-color: var(--border-primary); background-color: var(--bg-primary); color: var(--text-primary);"
					onclick={() => {
						if (selectedUser)
							api.POST('/auth/password/request-reset', { body: { email: selectedUser?.email } });
					}}>Reset</button
				>
			</div>
			<select class="mb-3 border-2 px-3 py-1" style="border-color: var(--border-primary); background-color: var(--bg-primary); color: var(--text-primary);" name="role" id="role" value={selectedUser?.role}>
				<option id="userRole" value="member">Member</option>
				<option id="adminRole" value="admin">Admin</option>
			</select>
			<button
				class="m-5 mb-0 cursor-pointer rounded border-2 px-4 py-2 font-bold hover:opacity-80"
				style="border-color: var(--border-primary); background-color: var(--bg-primary); color: var(--text-primary);"
				onclick={() => console.log('TODO: not yet implemented')}>Save</button
			>
		</form>
	</div>
</div>
