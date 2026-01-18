<script lang="ts">
	import { onDestroy } from 'svelte';
	import EmptyState from './EmptyState.svelte';
	import ImageList from './ImageList.svelte';
	import MergeOptions from './MergeOptions.svelte';
	import Preview from './Preview.svelte';
	import ErrorDialog from './ErrorDialog.svelte';
	import { ACCEPT_STRING } from '$lib/utils/formats';
	import { mergeImages as workerMerge } from '$lib/utils/workerManager';
	import { mergerState } from '$lib/mergerState.svelte';
	import { goto } from '$app/navigation';
	import { resolve } from '$app/paths';

	// File input ref for "Add More"
	let addMoreInput: HTMLInputElement | undefined = $state();

	onDestroy(() => {
		// Note: We do NOT revoke URLs here anymore because state is global.
		// We only revoke when explicitly removing images or resetting.
	});

	function handleFilesAdded(files: File[]) {
		const { error } = mergerState.addFiles(files);
		if (error) {
			mergerState.setError(error);
		}
	}

	function handleEmptyStateError(message: string) {
		mergerState.setError(message);
	}

	function handleReorder(newImages: any[]) {
		// Using any[] to match whatever ImageList emits, but ideally should be ImageFile[]
		mergerState.reorderImages(newImages);
	}

	function handleRemove(id: string) {
		mergerState.removeImage(id);
	}

	function handleDirectionChange(d: any) {
		mergerState.setDirection(d);
	}

	function handleBackgroundChange(bg: any) {
		mergerState.setBackground(bg);
	}

	async function handleMerge() {
		if (!mergerState.canMerge) return;

		// Set processing state
		mergerState.setMergeState({ status: 'processing', stage: 'Starting...', percent: 0 });

		const files = mergerState.images.map((img) => img.file);
		const options = { direction: mergerState.direction, background: mergerState.background };

		workerMerge(files, options, {
			async onSuccess(result) {
				const blob = new Blob([result.bytes], { type: result.mime });
				const url = URL.createObjectURL(blob);
				mergerState.setMergeState({
					status: 'success',
					blob,
					url,
					width: result.width,
					height: result.height
				});

				// Navigate to preview page
				await goto(resolve('/preview'));
				return;
			},
			onError: (error) => {
				mergerState.setMergeState({ status: 'error', error });
			}
		});
	}

	function handleErrorDismiss() {
		mergerState.setMergeState({ status: 'idle' });
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
		{#if mergerState.images.length === 0}
			<EmptyState onFilesAdded={handleFilesAdded} onError={handleEmptyStateError} />
		{:else}
			<ImageList images={mergerState.images} onRemove={handleRemove} onReorder={handleReorder} />

			<!-- Add more button -->
			<input
				bind:this={addMoreInput}
				class="hidden"
				accept={ACCEPT_STRING}
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
			background={mergerState.background}
			direction={mergerState.direction}
			onBackgroundChange={handleBackgroundChange}
			onDirectionChange={handleDirectionChange}
		/>

		<!-- Merge button -->
		<button
			class="btn preset-filled-primary-500 w-full text-lg py-3"
			data-testid="merge-button"
			disabled={!mergerState.canMerge || mergerState.isProcessing}
			onclick={handleMerge}
		>
			{#if mergerState.isProcessing}
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
			{:else if mergerState.images.length < 2}
				Add at least 2 images
			{:else}
				Merge {mergerState.images.length} Images
			{/if}
		</button>

		{#if mergerState.isProcessing}
			<div class="space-y-2">
				<div class="h-2 bg-surface-200 dark:bg-surface-700 rounded-full overflow-hidden">
					<div style:width="100%" class="h-full bg-primary-500 rounded-full animate-pulse"></div>
				</div>
				<p class="text-sm text-center text-surface-500">Processing images...</p>
			</div>
		{/if}
	</div>

	<!-- Right column: Preview (Optional now, maybe show placeholder or mini-preview) -->
	<!-- The user requested "show 'preview' in a new page". 
         They said "current preview is too small".
         We can either remove the preview from here entirely, or keep a small one.
         Given the request implies REPLACING the small preview workflow with a full page one,
         OR just providing a full page option. 
         However, the prompt says "show 'preview' in a new page... notice current preview is too small".
         It strongly suggests moving the preview experience to a new page.
         
         If we remove it here, the right column becomes empty or we need to redesign.
         Let's keep the right column as a "Quick Preview" or just remove it if we auto-navigate.
         
         Wait, I auto-navigated in handleMerge: `goto(\`${base}/preview\`);`
         So once merged, they go to the new page. 
         But what if they return? 
         When they return, they might want to see the result without merging again.
         
         If `mergerState.hasResult` is true, we could show a "View Result" button or a small preview.
         Let's keep a small preview here but maybe simplified, or just a "View Full Result" card.
    -->
	<div class="lg:sticky lg:top-4 lg:self-start">
		{#if mergerState.hasResult && mergerState.mergeState.status === 'success'}
			<!-- Mini preview with link to full page -->
			<div class="card p-4 space-y-4 bg-surface-100 dark:bg-surface-800">
				<h3 class="font-medium">Result Ready</h3>
				<div
					class="max-h-[200px] overflow-hidden rounded border border-surface-200 dark:border-surface-700 relative"
				>
					<img
						class="max-w-full h-auto opacity-50"
						alt="Merged result preview"
						src={mergerState.mergeState.url}
					/>
					<div class="absolute inset-0 flex items-center justify-center">
						<button
							class="btn preset-filled-secondary-500"
							onclick={async () => await goto(resolve('/preview'))}
						>
							View Full Result
						</button>
					</div>
				</div>
			</div>
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
{#if mergerState.hasError && mergerState.mergeState.status === 'error'}
	<ErrorDialog error={mergerState.mergeState.error} onDismiss={handleErrorDismiss} />
{/if}
