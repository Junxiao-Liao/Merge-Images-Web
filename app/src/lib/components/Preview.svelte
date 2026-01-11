<script lang="ts">
	import { downloadBlob } from '$lib/utils/download';

	interface Props {
		blob: Blob;
		url: string;
		width: number;
		height: number;
	}

	let { blob, url, width, height }: Props = $props();

	let showFallbackMessage = $state(false);

	function handleDownload() {
		const result = downloadBlob(blob, 'merged.png');
		if (result.usedFallback) {
			showFallbackMessage = true;
		}
	}

	function formatDimensions(w: number, h: number): string {
		return `${w.toLocaleString()} x ${h.toLocaleString()} pixels`;
	}

	function formatFileSize(bytes: number): string {
		if (bytes < 1024) return `${bytes} B`;
		if (bytes < 1024 * 1024) return `${(bytes / 1024).toFixed(1)} KB`;
		return `${(bytes / (1024 * 1024)).toFixed(1)} MB`;
	}
</script>

<div class="card p-4 space-y-4 bg-surface-100 dark:bg-surface-800">
	<div class="flex items-center justify-between">
		<h3 class="font-medium text-surface-900 dark:text-surface-50">Preview</h3>
		<div class="text-sm text-surface-500">
			{formatDimensions(width, height)} - {formatFileSize(blob.size)}
		</div>
	</div>

	<!-- Scrollable preview area -->
	<div
		class="max-h-[400px] overflow-auto rounded border border-surface-200 dark:border-surface-700"
	>
		<img class="max-w-full h-auto" alt="Merged result" src={url} />
	</div>

	<!-- Download button -->
	<button class="btn preset-filled-primary-500 w-full" onclick={handleDownload}>
		<svg class="w-5 h-5 mr-2" fill="none" stroke="currentColor" viewBox="0 0 24 24">
			<path
				d="M4 16v1a3 3 0 003 3h10a3 3 0 003-3v-1m-4-4l-4 4m0 0l-4-4m4 4V4"
				stroke-linecap="round"
				stroke-linejoin="round"
				stroke-width="2"
			/>
		</svg>
		Download merged.png
	</button>

	{#if showFallbackMessage}
		<p class="text-sm text-surface-500 text-center">
			Image opened in new tab. Long press to save on iOS.
		</p>
	{/if}
</div>
