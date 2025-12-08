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
	class="border-border-primary bg-bg-primary w-70 border-l-2 text-center {selectedUser
		? 'flex'
		: 'hidden'} flex-col items-center p-6"
>
	<span class="text-text-primary mb-1 text-xl font-bold">Profile</span>
	<div class="flex flex-col justify-items-center">
		<form class="mb-4 flex flex-col items-center px-8 pt-2 pb-8">
			<img class="h-50 w-50" src={PlaceHolderImage} alt="User Profile" />
			<input
				class="border-border-primary bg-bg-primary text-text-primary mb-2 border-2 px-3 py-1"
				id="fname"
				type="text"
				value={selectedUser?.first_name}
				required
				placeholder="First Name"
			/>
			<input
				class="border-border-primary bg-bg-primary text-text-primary mb-2 border-2 px-3 py-1"
				id="lname"
				type="text"
				value={selectedUser?.last_name}
				required
				placeholder="Last Name"
			/>
			<input
				class="border-border-primary bg-bg-primary text-text-primary mb-2 border-2 px-3 py-1"
				id="email"
				type="text"
				value={selectedUser?.email}
				required
				placeholder="Email"
			/>
			<div class="flex flex-col gap-4 md:flex-row">
				<input
					class="border-border-secondary bg-bg-secondary text-text-secondary mb-2 w-[60%] cursor-not-allowed border-2 px-3 py-1 select-none"
					id="password"
					type="text"
					value="*******"
					disabled
				/>
				<button
					class="border-border-primary bg-bg-primary text-text-primary mb-2 cursor-pointer rounded border-2 px-4 py-2 font-bold hover:opacity-80"
					onclick={() => {
						if (selectedUser)
							api.POST('/auth/password/request-reset', { body: { email: selectedUser?.email } });
					}}>Reset</button
				>
			</div>
			<select
				class="border-border-primary bg-bg-primary text-text-primary mb-3 border-2 px-3 py-1"
				name="role"
				id="role"
				value={selectedUser?.role}
			>
				<option id="userRole" value="member">Member</option>
				<option id="adminRole" value="admin">Admin</option>
			</select>
			<button
				class="border-border-primary bg-bg-primary text-text-primary m-5 mb-0 cursor-pointer rounded border-2 px-4 py-2 font-bold hover:opacity-80"
				onclick={() => console.log('TODO: not yet implemented')}>Save</button
			>
		</form>
	</div>
</div>
