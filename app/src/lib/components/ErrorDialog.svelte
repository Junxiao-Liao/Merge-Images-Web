<script lang="ts">
	import type { MergeError } from '$lib/types';

	interface Props {
		error: MergeError | null;
		onDismiss: () => void;
	}

	let { error, onDismiss }: Props = $props();

	function getErrorTitle(code: string): string {
		const titles: Record<string, string> = {
			DECODE_FAILED: 'Image Decode Failed',
			NO_IMAGES: 'No Images',
			INTERNAL_ERROR: 'Unexpected Error',
			UNSUPPORTED_FORMAT: 'Unsupported Format'
		};
		return titles[code] || 'Error';
	}

	function getErrorSuggestion(err: MergeError): string {
		if (err.code === 'DECODE_FAILED' && err.details?.fileIndex !== undefined) {
			return `Failed to decode image #${err.details.fileIndex + 1}. The file may be corrupted or in an unsupported format.`;
		}

		return err.message;
	}
</script>

{#if error}
	<!-- Backdrop -->
	<div
		class="fixed inset-0 bg-black/50 z-40"
		aria-label="Close dialog"
		onclick={onDismiss}
		onkeydown={(e) => e.key === 'Escape' && onDismiss()}
		role="button"
		tabindex="-1"
	></div>

	<!-- Dialog -->
	<div
		class="fixed left-1/2 top-1/2 -translate-x-1/2 -translate-y-1/2 z-50 w-full max-w-md"
		aria-labelledby="error-title"
		aria-modal="true"
		data-testid="error-dialog"
		role="dialog"
	>
		<div class="card p-6 bg-surface-50 dark:bg-surface-800 shadow-xl">
			<h2 id="error-title" class="text-lg font-bold text-error-500 flex items-center gap-2">
				<svg class="w-6 h-6" fill="none" stroke="currentColor" viewBox="0 0 24 24">
					<path
						d="M12 9v2m0 4h.01m-6.938 4h13.856c1.54 0 2.502-1.667 1.732-3L13.732 4c-.77-1.333-2.694-1.333-3.464 0L3.34 16c-.77 1.333.192 3 1.732 3z"
						stroke-linecap="round"
						stroke-linejoin="round"
						stroke-width="2"
					/>
				</svg>
				{getErrorTitle(error.code)}
			</h2>

			<p class="mt-4 text-surface-600 dark:text-surface-300">
				{getErrorSuggestion(error)}
			</p>

			<div class="mt-6 flex justify-end">
				<button class="btn preset-filled-surface-200-800" onclick={onDismiss}> Close </button>
			</div>
		</div>
	</div>
{/if}
