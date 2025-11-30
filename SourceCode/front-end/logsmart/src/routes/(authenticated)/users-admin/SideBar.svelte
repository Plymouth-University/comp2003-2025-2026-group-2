<script lang="ts">
	import { api } from '$lib/api';
	import type { Member } from './+page.svelte';

	const { setSelectedUser, selectedUser } = $props<{
		setSelectedUser: (email: string | null) => void;
		selectedUser: Member | null;
	}>();
</script>

<div
	id="userSidebar"
	class="w-70 border-l-2 bg-white text-center {selectedUser
		? 'flex'
		: 'hidden'} flex-col items-center p-6"
	style="border-color: #000100;"
>
    <span class="text-xl font-bold mb-1">Profile</span>
	<div class="flex flex-col justify-items-center">
		<form class="mb-4 px-8 pt-2 pb-8 flex flex-col items-center">
			<img class="w-50 h-50" src="src/lib/assets/placeholder.png" alt="User Profile" />
            <input class="mb-2" id="fname" type="text" value={selectedUser?.first_name} required placeholder="First Name"/>
            <input class="mb-2" id="lname" type="text" value={selectedUser?.last_name} required placeholder="Last Name"/>
			<input class="mb-2" id="email" type="text" value={selectedUser?.email} required placeholder="Email"/>
			<div class="flex flex-col gap-4 md:flex-row">
				<input
					class="mb-2 cursor-not-allowed bg-gray-200 select-none"
					id="password"
					type="text"
					value="*******"
					style="width:60%; -webkit-user-select: none; user-select: none; -ms-user-select: none;"
					disabled
				/>
				<button
					class="mb-2 cursor-pointer rounded border bg-gray-300 px-4 py-2 font-bold text-black hover:bg-slate-700 hover:text-white"
					onclick={() => {
						if (selectedUser)
							api.POST('/auth/password/request-reset', { body: { email: selectedUser?.email } });
					}}>Reset</button
				>
			</div>
			<select class="mb-3" name="role" id="role" value={selectedUser?.role}>
				<option id="userRole" value="member">Member</option>
				<option id="adminRole" value="admin">Admin</option>
			</select>
			<button
				class="m-5 mb-0 cursor-pointer rounded border bg-gray-300 px-4 py-2 font-bold text-black hover:bg-slate-700 hover:text-white"
				onclick={() => console.log('TODO: not yet implemented')}>Save</button
			>
		</form>
	</div>
</div>
