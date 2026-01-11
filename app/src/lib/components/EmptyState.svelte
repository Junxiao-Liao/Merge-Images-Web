<script lang="ts">
	interface Props {
		onFilesAdded: (addedFiles: File[]) => void;
	}

	let { onFilesAdded }: Props = $props();

	let isDragging = $state(false);
	let inputRef: HTMLInputElement | undefined = $state();

	function handleFileChange(event: Event) {
		const input = event.target as HTMLInputElement;
		if (input.files && input.files.length > 0) {
			onFilesAdded(Array.from(input.files));
			input.value = ''; // Reset for next selection
		}
	}

	function handleDrop(event: DragEvent) {
		event.preventDefault();
		isDragging = false;
		if (event.dataTransfer?.files && event.dataTransfer.files.length > 0) {
			const files = Array.from(event.dataTransfer.files).filter((f) => f.type.startsWith('image/'));
			if (files.length > 0) {
				onFilesAdded(files);
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
		accept="image/png,image/jpeg,image/gif,image/webp"
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
			<p class="text-sm text-surface-500 mt-1">PNG, JPG, GIF, WebP supported</p>
		</div>
	</div>
</div>
