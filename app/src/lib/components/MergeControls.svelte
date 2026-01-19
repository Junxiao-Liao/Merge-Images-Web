<script lang="ts">
	interface Props {
		canMerge: boolean;
		isProcessing: boolean;
		imageCount: number;
	}

	let { canMerge, isProcessing, imageCount }: Props = $props();
</script>

<div class="space-y-4 max-w-sm mx-auto">
	<button
		class="btn preset-filled-primary-500 w-full text-base sm:text-lg py-2 sm:py-3"
		disabled={!canMerge || isProcessing}
	>
		{#if isProcessing}
			<svg class="animate-spin -ml-1 mr-2 h-5 w-5" fill="none" viewBox="0 0 24 24">
				<circle class="opacity-25" cx="12" cy="12" r="10" stroke="currentColor" stroke-width="4"
				></circle>
				<path
					class="opacity-75"
					d="M4 12a8 8 0 018-8V0C5.373 0 0 5.373 0 12h4zm2 5.291A7.962 7.962 0 014 12H0c0 3.042 1.135 5.824 3 7.938l3-2.647z"
					fill="currentColor"
				></path>
			</svg>
			Merging...
		{:else}
			Merge {imageCount} Images
		{/if}
	</button>

	{#if isProcessing}
		<div class="space-y-2">
			<div class="h-2 bg-surface-200 dark:bg-surface-700 rounded-full overflow-hidden">
				<div style:width="100%" class="h-full bg-primary-500 rounded-full animate-pulse"></div>
			</div>
			<p class="text-sm text-center text-surface-500">Processing images...</p>
		</div>
	{/if}

	{#if imageCount < 2 && imageCount > 0}
		<p class="text-sm text-center text-surface-500">Add at least 2 images to merge</p>
	{/if}
</div>
