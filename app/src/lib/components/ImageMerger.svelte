<script lang="ts">
	import { onDestroy } from 'svelte';
	import EmptyState from './EmptyState.svelte';
	import ImageList from './ImageList.svelte';
	import MergeOptions from './MergeOptions.svelte';
	import Preview from './Preview.svelte';
	import ErrorDialog from './ErrorDialog.svelte';
	import type { ImageFile, MergeState, Direction, BackgroundColor } from '$lib/types';
	import { DEFAULT_OPTIONS } from '$lib/workers/types';
	import { createImageFile, revokeImageFile, revokeAllImageFiles } from '$lib/utils/thumbnails';
	import { mergeImages as workerMerge } from '$lib/utils/workerManager';

	// Image state
	let images = $state<ImageFile[]>([]);

	// Merge options
	let direction = $state<Direction>(DEFAULT_OPTIONS.direction);
	let background = $state<BackgroundColor>(DEFAULT_OPTIONS.background);

	// Processing state
	let mergeState = $state<MergeState>({ status: 'idle' });

	// File input ref for "Add More"
	let addMoreInput: HTMLInputElement | undefined = $state();

	// Derived state
	let canMerge = $derived(images.length >= 2 && mergeState.status !== 'processing');
	let isProcessing = $derived(mergeState.status === 'processing');
	let hasResult = $derived(mergeState.status === 'success');
	let hasError = $derived(mergeState.status === 'error');

	onDestroy(() => {
		// Cleanup object URLs
		revokeAllImageFiles(images);
		if (mergeState.status === 'success') {
			URL.revokeObjectURL(mergeState.url);
		}
	});

	function handleFilesAdded(files: File[]) {
		// Filter for valid image types
		const imageFiles = files.filter((f) =>
			['image/png', 'image/jpeg', 'image/gif', 'image/webp'].includes(f.type)
		);

		if (imageFiles.length === 0) return;

		// Create ImageFile objects
		const newImages = imageFiles.map(createImageFile);
		images = [...images, ...newImages];
	}

	function handleReorder(newImages: ImageFile[]) {
		images = newImages;
	}

	function handleRemove(id: string) {
		const image = images.find((img) => img.id === id);
		if (image) {
			revokeImageFile(image);
		}
		images = images.filter((img) => img.id !== id);
	}

	function handleDirectionChange(d: Direction) {
		direction = d;
	}

	function handleBackgroundChange(bg: BackgroundColor) {
		background = bg;
	}

	function handleMerge() {
		if (!canMerge) return;

		// Clear previous result
		if (mergeState.status === 'success') {
			URL.revokeObjectURL(mergeState.url);
		}

		mergeState = { status: 'processing', stage: 'Starting...', percent: 0 };

		const files = images.map((img) => img.file);
		const options = { direction, background };

		workerMerge(files, options, {
			onSuccess: (result) => {
				const blob = new Blob([result.bytes], { type: result.mime });
				const url = URL.createObjectURL(blob);
				mergeState = {
					status: 'success',
					blob,
					url,
					width: result.width,
					height: result.height
				};
			},
			onError: (error) => {
				mergeState = { status: 'error', error };
			}
		});
	}

	function handleErrorDismiss() {
		mergeState = { status: 'idle' };
	}

	function handleAddMoreChange(event: Event) {
		const input = event.target as HTMLInputElement;
		if (input.files && input.files.length > 0) {
			handleFilesAdded(Array.from(input.files));
			input.value = ''; // Reset for next selection
		}
	}

	function handleAddMoreClick() {
		addMoreInput?.click();
	}
</script>

<div class="grid grid-cols-1 lg:grid-cols-2 gap-6">
	<!-- Left column: Images + Options -->
	<div class="space-y-6">
		{#if images.length === 0}
			<EmptyState onFilesAdded={handleFilesAdded} />
		{:else}
			<ImageList {images} onRemove={handleRemove} onReorder={handleReorder} />

			<!-- Add more button -->
			<input
				bind:this={addMoreInput}
				class="hidden"
				accept="image/png,image/jpeg,image/gif,image/webp"
				multiple
				onchange={handleAddMoreChange}
				type="file"
			/>
			<button class="btn preset-tonal-primary w-full" onclick={handleAddMoreClick}>
				<svg class="w-5 h-5 mr-2" fill="none" stroke="currentColor" viewBox="0 0 24 24">
					<path
						d="M12 6v6m0 0v6m0-6h6m-6 0H6"
						stroke-linecap="round"
						stroke-linejoin="round"
						stroke-width="2"
					/>
				</svg>
				Add More Images
			</button>
		{/if}

		<MergeOptions
			{background}
			{direction}
			onBackgroundChange={handleBackgroundChange}
			onDirectionChange={handleDirectionChange}
		/>

		<!-- Merge button -->
		<button
			class="btn preset-filled-primary-500 w-full text-lg py-3"
			disabled={!canMerge || isProcessing}
			onclick={handleMerge}
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
			{:else if images.length < 2}
				Add at least 2 images
			{:else}
				Merge {images.length} Images
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
	</div>

	<!-- Right column: Preview -->
	<div class="lg:sticky lg:top-4 lg:self-start">
		{#if hasResult && mergeState.status === 'success'}
			<Preview
				blob={mergeState.blob}
				height={mergeState.height}
				url={mergeState.url}
				width={mergeState.width}
			/>
		{:else}
			<div
				class="card p-8 bg-surface-100 dark:bg-surface-800 flex items-center justify-center min-h-[300px]"
			>
				<div class="text-center text-surface-500">
					<svg
						class="w-16 h-16 mx-auto mb-4 opacity-50"
						fill="none"
						stroke="currentColor"
						viewBox="0 0 24 24"
					>
						<path
							d="M4 16l4.586-4.586a2 2 0 012.828 0L16 16m-2-2l1.586-1.586a2 2 0 012.828 0L20 14m-6-6h.01M6 20h12a2 2 0 002-2V6a2 2 0 00-2-2H6a2 2 0 00-2 2v12a2 2 0 002 2z"
							stroke-linecap="round"
							stroke-linejoin="round"
							stroke-width="2"
						/>
					</svg>
					<p>Preview will appear here</p>
				</div>
			</div>
		{/if}
	</div>
</div>

<!-- Error dialog -->
{#if hasError && mergeState.status === 'error'}
	<ErrorDialog error={mergeState.error} onDismiss={handleErrorDismiss} />
{/if}
