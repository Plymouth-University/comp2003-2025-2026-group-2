<script lang="ts">
	let { show = false }: { show?: boolean } = $props();

	const acceptCookies = () => {
		document.cookie = 'cookies_accepted=true; path=/; max-age=31536000';
		show = false;
	};

	const dismissNotice = () => {
		document.cookie = 'cookies_notice_dismissed=true; path=/; max-age=31536000';
		show = false;
	};
</script>

{#if show}
	<div
		class="fixed right-0 bottom-0 left-0 z-1000 border-t-2 border-border-primary bg-bg-secondary px-6 py-6 shadow-[0_-2px_10px_rgba(0,0,0,0.1)]"
	>
		<div class="mx-auto max-w-4xl text-left">
			<h3 class="mb-3 text-lg text-text-primary">Cookie Notice</h3>
			<p class="mb-3 text-sm leading-relaxed text-text-secondary">
				LogSmart.app uses a single essential authentication cookie to keep you signed in and secure.
				We do not use analytics or marketing cookies.
			</p>
			<p class="mb-4 text-sm leading-relaxed text-text-secondary">
				You can learn more about this cookie and how it is used in our Cookie Policy.
			</p>
			<div class="mt-4 flex gap-4">
				<a
					href="/cookie-policy"
					class="rounded-lg border border-border-primary bg-transparent px-6 py-2 text-sm font-medium text-text-primary transition-colors hover:border-text-primary"
				>
					View Policy
				</a>
				<button
					onclick={dismissNotice}
					class="bg-primary-color rounded-lg border-none px-6 py-2 text-sm font-medium text-white transition-opacity hover:opacity-90"
				>
					Dismiss
				</button>
			</div>
		</div>
	</div>
{/if}
