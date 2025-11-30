<script lang="ts">
	const { item, setSelectedUser } = $props<{
		item: {
			email: string;
			first_name: string;
			last_name: string;
			role: string;
		};
		setSelectedUser: (email: string) => void;
	}>();
</script>

<button
	onclick={() => setSelectedUser(item.email)}
	class="mb-2 flex h-20 flex-auto flex-row content-between overflow-x-auto border-2 bg-white hover:bg-gray-100 active:bg-gray-200"
	style="margin-top: -5px; display:flex;"
>
	<div class="userBox-rows">{item.email}</div>
	<div class="userBox-rows" style="flex:1">{item.first_name} {item.last_name}</div>
	<div class="userBox-rows">{item.role}</div>
	<div class="userBox-rows">
		<a class="hover:underline focus:underline" href="#remove-user">Remove</a>
	</div>
</button>

<style>
	.userBox-rows {
		padding: 1%;
		padding-bottom: 1%;
		padding-left: 5%;
		padding-right: 5%;
		padding-top: 1.5%;
	}
</style>
