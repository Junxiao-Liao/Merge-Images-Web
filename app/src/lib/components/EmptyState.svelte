<script lang="ts">
	import { ACCEPT_STRING, categorizeFiles } from '$lib/utils/formats';

	interface Props {
		onFilesAdded: (addedFiles: File[]) => void;
		onError?: (message: string) => void;
	}

	let { onFilesAdded, onError }: Props = $props();

	let isDragging = $state(false);
	let inputRef: HTMLInputElement | undefined = $state();

	function handleFileChange(event: Event) {
		const input = event.target as HTMLInputElement;
		if (input.files && input.files.length > 0) {
			const { supported, heicFiles } = categorizeFiles(Array.from(input.files));

			// Report HEIC files early rejection
			if (heicFiles.length > 0) {
				const names = heicFiles.map((f) => f.name).join(', ');
				onError?.(
					`HEIC/HEIF format is not supported: ${names}. Please convert to PNG or JPEG first.`
				);
			}

			if (supported.length > 0) {
				onFilesAdded(supported);
			}
			input.value = ''; // Reset for next selection
		}
	}

	function handleDrop(event: DragEvent) {
		event.preventDefault();
		isDragging = false;
		if (event.dataTransfer?.files && event.dataTransfer.files.length > 0) {
			const { supported, heicFiles } = categorizeFiles(Array.from(event.dataTransfer.files));

			// Report HEIC files early rejection
			if (heicFiles.length > 0) {
				const names = heicFiles.map((f) => f.name).join(', ');
				onError?.(
					`HEIC/HEIF format is not supported: ${names}. Please convert to PNG or JPEG first.`
				);
			}

			if (supported.length > 0) {
				onFilesAdded(supported);
			}
		}
	}

	function handleDragOver(event: DragEvent) {
		event.preventDefault();
		isDragging = true;
	}

	function handleDragLeave(event: DragEvent) {
		event.preventDefault();
		isDragging = false;
	}

	function handleClick() {
		inputRef?.click();
	}

	function handleKeyDown(event: KeyboardEvent) {
		if (event.key === 'Enter' || event.key === ' ') {
			event.preventDefault();
			inputRef?.click();
		}
	}
</script>

<div
	class="border-2 border-dashed rounded-lg p-8 text-center transition-colors cursor-pointer {isDragging
		? 'border-primary-500 bg-primary-500/10'
		: 'border-surface-300 dark:border-surface-600 hover:border-primary-500'}"
	data-testid="drop-zone"
	onclick={handleClick}
	ondragleave={handleDragLeave}
	ondragover={handleDragOver}
	ondrop={handleDrop}
	onkeydown={handleKeyDown}
	role="button"
	tabindex="0"
>
	<input
		bind:this={inputRef}
		class="hidden"
		accept={ACCEPT_STRING}
		data-testid="file-input"
		multiple
		onchange={handleFileChange}
		type="file"
	/>

	<div class="flex flex-col items-center gap-4">
		<svg class="w-16 h-16 text-surface-400" fill="none" stroke="currentColor" viewBox="0 0 24 24">
			<path
				d="M4 16l4.586-4.586a2 2 0 012.828 0L16 16m-2-2l1.586-1.586a2 2 0 012.828 0L20 14m-6-6h.01M6 20h12a2 2 0 002-2V6a2 2 0 00-2-2H6a2 2 0 00-2 2v12a2 2 0 002 2z"
				stroke-linecap="round"
				stroke-linejoin="round"
				stroke-width="2"
			/>
		</svg>
		<div>
			<p class="text-lg font-medium text-surface-700 dark:text-surface-200">
				Drop images here or click to select
			</p>
			<p class="text-sm text-surface-500 mt-1">PNG, JPG, GIF, WebP, TIFF supported</p>
		</div>
	</div>
</div>
