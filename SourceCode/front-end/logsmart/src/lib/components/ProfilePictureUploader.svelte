<script lang="ts">
	import { onDestroy, onMount, tick } from 'svelte';

	let {
		currentPictureUrl = null as string | null,
		onUploadComplete,
		onDeleteComplete,
		disabled = false,
		firstName = '',
		lastName = '',
		triggerOnImageClick = false,
		showUploadButton = true,
		targetUserEmail = ''
	}: {
		currentPictureUrl?: string | null;
		onUploadComplete?: (url: string) => void;
		onDeleteComplete?: () => void;
		disabled?: boolean;
		firstName?: string;
		lastName?: string;
		triggerOnImageClick?: boolean;
		showUploadButton?: boolean;
		targetUserEmail?: string;
	} = $props();
	const uid = $props.id();

	let fileInput: HTMLInputElement | null = $state(null);
	let imageElement: HTMLImageElement | null = $state(null);
	let cropperContainer: HTMLDivElement | null = $state(null);
	type CropperConstructor = typeof import('cropperjs').default;
	let cropperConstructor: CropperConstructor | null = null;
	let cropper: InstanceType<CropperConstructor> | null = null;
	let isLoading = $state(false);
	let showCropper = $state(false);
	let errorMessage = $state('');

	function handlePictureClick() {
		if (!triggerOnImageClick || disabled || isLoading) return;
		fileInput?.click();
	}

	async function loadCropper() {
		if (cropperConstructor) return cropperConstructor;
		if (typeof window === 'undefined') return null;
		const module = await import('cropperjs');
		cropperConstructor = module.default;
		return cropperConstructor;
	}

	async function handleFileSelect(event: Event) {
		const target = event.target as HTMLInputElement;
		const file = target.files?.[0];
		if (!file) return;

		if (file.size > 10 * 1024 * 1024) {
			errorMessage = 'File too large. Maximum size is 10MB.';
			return;
		}

		if (!file.type.startsWith('image/')) {
			errorMessage = 'Please select an image file.';
			return;
		}

		errorMessage = '';

		const reader = new FileReader();
		reader.onload = async (e) => {
			showCropper = true;
			await tick();
			await loadCropper();

			if (imageElement && typeof e.target?.result === 'string') {
				imageElement.onload = () => {
					initCropper();
				};
				imageElement.src = e.target.result;
			}
		};
		reader.readAsDataURL(file);
		target.value = '';
	}

	function initCropper() {
		if (cropper) {
			cropper.destroy();
		}

		if (!cropperContainer || !imageElement || !cropperConstructor) return;
		cropperContainer.querySelectorAll('cropper-canvas').forEach((node) => node.remove());
		if (typeof window === 'undefined') return;
		const template =
			'<cropper-canvas background>' +
			'<cropper-image rotatable scalable skewable translatable></cropper-image>' +
			'<cropper-selection action=move initial-coverage="0.8" movable resizable>' +
			'<cropper-grid role="grid" bordered covered></cropper-grid>' +
			'<cropper-handle action="n-resize"></cropper-handle>' +
			'<cropper-handle action="e-resize"></cropper-handle>' +
			'<cropper-handle action="s-resize"></cropper-handle>' +
			'<cropper-handle action="w-resize"></cropper-handle>' +
			'<cropper-handle action="ne-resize"></cropper-handle>' +
			'<cropper-handle action="nw-resize"></cropper-handle>' +
			'<cropper-handle action="se-resize"></cropper-handle>' +
			'<cropper-handle action="sw-resize"></cropper-handle>' +
			'</cropper-selection>' +
			'</cropper-canvas>';

		cropper = new cropperConstructor(imageElement, {
			container: cropperContainer,
			template
		});

		const image = cropper.getCropperImage?.();
		if (image) {
			image.translatable = true;
		}

		const selection = cropper.getCropperSelection?.();
		if (selection) {
			selection.aspectRatio = 1;
			selection.initialAspectRatio = 1;
			selection.initialCoverage = 0.8;
			selection.movable = true;
			selection.resizable = true;
			selection.zoomable = true;
			selection.active = true;
			selection.$center?.();
			selection.$render?.();
		}
	}

	async function handleSave() {
		if (!cropper) return;

		isLoading = true;
		errorMessage = '';

		try {
			const selection = cropper?.getCropperSelection?.();
			if (!selection) {
				throw new Error('Cropper selection not available');
			}

			const canvas = await selection.$toCanvas({
				width: 512,
				height: 512,
				beforeDraw(context: CanvasRenderingContext2D) {
					context.imageSmoothingEnabled = true;
					context.imageSmoothingQuality = 'high';
				}
			});

			const blob = await new Promise<Blob | null>((resolve) =>
				canvas.toBlob(resolve, 'image/webp', 0.85)
			);
			if (!blob) {
				throw new Error('Failed to process image');
			}

			const uploadUrl = targetUserEmail
				? `/api/auth/profile-picture?email=${encodeURIComponent(targetUserEmail)}`
				: '/api/auth/profile-picture';
			const response = await fetch(uploadUrl, {
				method: 'POST',
				body: blob,
				headers: {
					'Content-Type': 'image/webp'
				}
			});

			if (!response.ok) {
				const err = await response.json();
				throw new Error(err.error || 'Failed to upload picture');
			}

			const data = await response.json();
			showCropper = false;
			cropper?.destroy();
			cropper = null;

			if (onUploadComplete) {
				onUploadComplete(data.profile_picture_url);
			}
		} catch (err) {
			errorMessage = err instanceof Error ? err.message : 'Failed to upload picture';
		} finally {
			isLoading = false;
		}
	}

	function handleCancel() {
		showCropper = false;
		if (cropper) {
			cropper.destroy();
			cropper = null;
		}
		if (imageElement) {
			imageElement.src = '';
		}
	}

	async function handleDelete() {
		if (!confirm('Are you sure you want to delete your profile picture?')) return;

		isLoading = true;
		errorMessage = '';

		try {
			const deleteUrl = targetUserEmail
				? `/api/auth/profile-picture?email=${encodeURIComponent(targetUserEmail)}`
				: '/api/auth/profile-picture';
			const response = await fetch(deleteUrl, {
				method: 'DELETE'
			});

			if (!response.ok) {
				const err = await response.json();
				throw new Error(err.error || 'Failed to delete picture');
			}

			if (onDeleteComplete) {
				onDeleteComplete();
			}
		} catch (err) {
			errorMessage = err instanceof Error ? err.message : 'Failed to delete picture';
		} finally {
			isLoading = false;
		}
	}

	onDestroy(() => {
		if (cropper) {
			cropper.destroy();
		}
	});

	onMount(() => {
		void loadCropper();
	});
</script>

<div class="profile-picture-uploader">
	{#if showCropper}
		<div class="cropper-modal">
			<div class="cropper-container" bind:this={cropperContainer}>
				<img bind:this={imageElement} src="" alt="Crop preview" class="cropper-image" />
			</div>
			<div class="cropper-actions">
				<button type="button" onclick={handleCancel} class="btn-cancel" disabled={isLoading}>
					Cancel
				</button>
				<button type="button" onclick={handleSave} class="btn-save" disabled={isLoading}>
					{isLoading ? 'Saving...' : 'Save'}
				</button>
			</div>
		</div>
	{:else}
		{#if triggerOnImageClick}
			<button
				type="button"
				class="current-picture clickable"
				onclick={handlePictureClick}
				disabled={disabled || isLoading}
			>
				{#if currentPictureUrl}
					<img src={currentPictureUrl} alt="Profile" class="profile-preview" />
				{:else}
					<div class="no-picture">
						<span class="initials">
							{firstName?.[0] || ''}{lastName?.[0] || ''}
						</span>
					</div>
				{/if}
				{#if !disabled}
					<div class="click-overlay">Change</div>
				{/if}
			</button>
		{:else}
			<div class="current-picture">
				{#if currentPictureUrl}
					<img src={currentPictureUrl} alt="Profile" class="profile-preview" />
				{:else}
					<div class="no-picture">
						<span class="initials">
							{firstName?.[0] || ''}{lastName?.[0] || ''}
						</span>
					</div>
				{/if}
			</div>
		{/if}

		{#if errorMessage}
			<p class="error">{errorMessage}</p>
		{/if}

		<div class="actions">
			<input
				type="file"
				accept="image/*"
				bind:this={fileInput}
				onchange={handleFileSelect}
				disabled={disabled || isLoading}
				class="file-input"
				id="{uid}-profile-picture-input"
			/>
			{#if showUploadButton}
				<label
					for="{uid}-profile-picture-input"
					class="btn-upload"
					class:disabled={disabled || isLoading}
				>
					{isLoading ? 'Uploading...' : currentPictureUrl ? 'Change Picture' : 'Upload Picture'}
				</label>
			{/if}

			{#if currentPictureUrl}
				<button
					type="button"
					onclick={handleDelete}
					class="btn-delete"
					disabled={disabled || isLoading}
				>
					Delete
				</button>
			{/if}
		</div>
	{/if}
</div>

<style>
	.profile-picture-uploader {
		display: flex;
		flex-direction: column;
		align-items: center;
		gap: 1rem;
	}

	.current-picture {
		width: 128px;
		height: 128px;
		border-radius: 50%;
		overflow: hidden;
		border: 3px solid var(--border-primary);
		position: relative;
	}

	.current-picture.clickable {
		cursor: pointer;
		padding: 0;
		background: transparent;
	}

	.click-overlay {
		position: absolute;
		inset: 0;
		display: flex;
		align-items: center;
		justify-content: center;
		background: rgba(17, 24, 39, 0.6);
		color: #f9fafb;
		font-weight: 600;
		font-size: 0.875rem;
		letter-spacing: 0.02em;
		opacity: 0;
		transition: opacity 0.2s ease;
		pointer-events: none;
	}

	.current-picture.clickable:hover .click-overlay,
	.current-picture.clickable:focus-visible .click-overlay {
		opacity: 1;
	}

	.profile-preview {
		width: 100%;
		height: 100%;
		object-fit: cover;
	}

	.no-picture {
		width: 100%;
		height: 100%;
		background: var(--bg-secondary);
		display: flex;
		align-items: center;
		justify-content: center;
	}

	.initials {
		font-size: 2.5rem;
		font-weight: bold;
		color: var(--text-secondary);
	}

	.cropper-modal {
		position: fixed;
		top: 0;
		left: 0;
		right: 0;
		bottom: 0;
		background: rgba(0, 0, 0, 0.8);
		display: flex;
		flex-direction: column;
		align-items: center;
		justify-content: center;
		z-index: 1000;
	}

	.cropper-container {
		max-width: 90vw;
		max-height: 70vh;
	}

	.cropper-image {
		max-width: 100%;
		max-height: 70vh;
	}

	.cropper-modal :global(cropper-canvas),
	.cropper-modal :global(cropper-image),
	.cropper-modal :global(cropper-selection) {
		display: block;
	}

	.cropper-modal :global(cropper-canvas) {
		width: 80vw;
		height: 70vh;
		max-width: 720px;
		max-height: 720px;
		background: #111827;
		border-radius: 8px;
		overflow: hidden;
	}

	.cropper-modal :global(cropper-image) {
		touch-action: none;
	}

	.cropper-modal :global(cropper-selection) {
		outline: 2px solid #f9fafb;
		cursor: move;
		touch-action: none;
		pointer-events: all;
	}

	.cropper-modal :global(cropper-grid) {
		pointer-events: none;
	}

	.cropper-modal :global(cropper-handle) {
		background: #f9fafb;
		border-radius: 999px;
		width: 12px;
		height: 12px;
		border: 2px solid #111827;
	}

	.cropper-actions {
		display: flex;
		gap: 1rem;
		margin-top: 1rem;
	}

	.file-input {
		display: none;
	}

	.actions {
		display: flex;
		gap: 0.5rem;
	}

	.btn-upload,
	.btn-delete,
	.btn-save,
	.btn-cancel {
		padding: 0.5rem 1rem;
		border: 2px solid var(--border-primary);
		background: var(--bg-primary);
		color: var(--text-primary);
		cursor: pointer;
		border-radius: 0.25rem;
		font-weight: 500;
	}

	.btn-upload:hover:not(.disabled),
	.btn-delete:hover:not(.disabled),
	.btn-save:hover:not(.disabled) {
		opacity: 0.8;
	}

	.btn-upload.disabled,
	.btn-cancel:disabled {
		opacity: 0.5;
		cursor: not-allowed;
	}

	.btn-delete {
		border-color: #ef4444;
		color: #ef4444;
	}

	.error {
		color: #ef4444;
		font-size: 0.875rem;
	}
</style>
