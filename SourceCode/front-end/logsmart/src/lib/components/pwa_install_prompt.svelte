<script lang="ts">
	import { onMount } from 'svelte';
	import { browser } from '$app/environment';

	let deferredPrompt: any = null;
	let showInstallPrompt = false;
	let isIOS = false;
	let isStandalone = false;

	onMount(() => {
		if (!browser) return;

		isStandalone = window.matchMedia('(display-mode: standalone)').matches;
		isIOS = /iPad|iPhone|iPod/.test(navigator.userAgent) && !(window as any).MSStream;

		window.addEventListener('beforeinstallprompt', (e) => {
			e.preventDefault();
			deferredPrompt = e;
			showInstallPrompt = true;
		});

		window.addEventListener('appinstalled', () => {
			showInstallPrompt = false;
			deferredPrompt = null;
		});

		if (isIOS && !isStandalone) {
			const dismissed = localStorage.getItem('ios-install-dismissed');
			if (!dismissed) {
				showInstallPrompt = true;
			}
		}
	});

	async function handleInstall() {
		if (!deferredPrompt) {
			if (isIOS) {
				localStorage.setItem('ios-install-dismissed', 'true');
				showInstallPrompt = false;
			}
			return;
		}

		deferredPrompt.prompt();
		const { outcome } = await deferredPrompt.userChoice;

		if (outcome === 'accepted') {
			showInstallPrompt = false;
		}

		deferredPrompt = null;
	}

	function dismiss() {
		showInstallPrompt = false;
		if (isIOS) {
			localStorage.setItem('ios-install-dismissed', 'true');
		}
	}
</script>

{#if showInstallPrompt && !isStandalone}
	<div class="fixed right-4 bottom-4 left-4 z-50 md:right-4 md:left-auto md:max-w-md">
		<div
			class="rounded-lg border border-gray-200 bg-white p-4 shadow-lg dark:border-gray-700 dark:bg-gray-800"
		>
			<div class="flex items-start gap-3">
				<div class="shrink-0">
					<svg class="h-8 w-8 text-blue-600" fill="none" stroke="currentColor" viewBox="0 0 24 24">
						<path
							stroke-linecap="round"
							stroke-linejoin="round"
							stroke-width="2"
							d="M12 18h.01M8 21h8a2 2 0 002-2V5a2 2 0 00-2-2H8a2 2 0 00-2 2v14a2 2 0 002 2z"
						></path>
					</svg>
				</div>
				<div class="flex-1">
					<h3 class="text-sm font-semibold text-gray-900 dark:text-white">Install LogSmart</h3>
					{#if isIOS}
						<p class="mt-1 text-xs text-gray-600 dark:text-gray-400">
							Tap the Share button <svg
								class="inline h-4 w-4"
								fill="currentColor"
								viewBox="0 0 20 20"
								><path
									d="M15 8a3 3 0 10-2.977-2.63l-4.94 2.47a3 3 0 100 4.319l4.94 2.47a3 3 0 10.895-1.789l-4.94-2.47a3.027 3.027 0 000-.74l4.94-2.47C13.456 7.68 14.19 8 15 8z"
								></path></svg
							> and select "Add to Home Screen"
						</p>
					{:else}
						<p class="mt-1 text-xs text-gray-600 dark:text-gray-400">
							Install LogSmart for quick access and offline use
						</p>
					{/if}
				</div>
				<button
					on:click={dismiss}
					class="shrink-0 text-gray-400 hover:text-gray-600 dark:hover:text-gray-300"
					aria-label="Dismiss"
				>
					<svg class="h-5 w-5" fill="none" stroke="currentColor" viewBox="0 0 24 24">
						<path
							stroke-linecap="round"
							stroke-linejoin="round"
							stroke-width="2"
							d="M6 18L18 6M6 6l12 12"
						></path>
					</svg>
				</button>
			</div>
			{#if !isIOS}
				<div class="mt-3 flex gap-2">
					<button
						on:click={handleInstall}
						class="flex-1 rounded bg-blue-600 px-4 py-2 text-sm font-medium text-white transition-colors hover:bg-blue-700"
					>
						Install
					</button>
					<button
						on:click={dismiss}
						class="flex-1 rounded bg-gray-200 px-4 py-2 text-sm font-medium text-gray-900 transition-colors hover:bg-gray-300 dark:bg-gray-700 dark:text-white dark:hover:bg-gray-600"
					>
						Later
					</button>
				</div>
			{/if}
		</div>
	</div>
{/if}
