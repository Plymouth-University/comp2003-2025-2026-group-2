<script lang="ts">
	let {
		upload = $bindable(''),
		placeholder = 'Upload Image',
		disabled = false
	}: {
		upload: string;
		placeholder: string;
		disabled?: boolean;
	} = $props();

	function handleFileUpload(event: Event) {
		const input = event.target as HTMLInputElement;
		if (input.files && input.files[0]) {
			const file = input.files[0];
			
			if (file.size > 5 * 1024 * 1024) {
				alert('File size exceeds 5MB limit. Please upload a smaller image.');
				return;
			}
			
			compressAndUploadImage(file);
		}
	}
	
	async function compressAndUploadImage(file: File) {
		try {
			const arrayBuffer = await file.arrayBuffer();
			const blob = new Blob([arrayBuffer], { type: 'image/webp' });
			
			const formData = new FormData();
			formData.append('image', blob, file.name);
			
			const response = await fetch('/api/images/upload', {
				method: 'POST',
				body: formData
			});
			
			if (response.ok) {
				const result = await response.json();
				upload = result.filename;
			} else {
				alert('Failed to upload image');
			}
		} catch (error) {
			console.error('Error uploading image:', error);
			alert('Error uploading image');
		}
	}
</script>

<div
	class="relative cursor-pointer border-2 px-4 py-2 rounded"
	class:border-2={!upload}
	class:border-primary={!upload}
	class:border-transparent={!!upload}
	style="border-color: var(--border-primary); color: var(--text-primary); background-color: var(--bg-primary);"
	onclick={() => {
		if (!upload && !disabled) {
			document.getElementById('image-input')?.click();
		}
	}}
	onkeydown={(e) => {
		if (e.key === 'Enter' && !upload && !disabled) {
			document.getElementById('image-input')?.click();
		}
	}}
	role="button"
	tabindex="0"
>
	<input
		id="image-input"
		type="file"
		accept="image/*"
		class="hidden"
		onchange={(e) => handleFileUpload(e)}
		onkeydown={(e) => {
			if (e.key === 'Enter') {
				document.getElementById('image-input')?.click();
			}
		}}
	{disabled}
	/>
	<div class="flex items-center gap-2">
		{#if upload}
			<div class="flex h-6 w-6 items-center justify-center">
				<span class="text-green-500">✓</span>
			</div>
			<span>Image uploaded</span>
		{:else}
			<div class="flex h-6 w-6 items-center justify-center">
				<span>📷</span>
			</div>
			<span>{placeholder}</span>
		{/if}
	</div>
</div>

<style>
	:global(.hidden) {
		display: none;
	}
</style>
