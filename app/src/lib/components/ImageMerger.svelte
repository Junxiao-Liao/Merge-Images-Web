<script lang="ts">
	import { onDestroy } from 'svelte';
	import EmptyState from './EmptyState.svelte';
	import ImageList from './ImageList.svelte';
	import MergeOptions from './MergeOptions.svelte';
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

	function handleOverlapSensitivityChange(value: number) {
		mergerState.setOverlapSensitivity(Math.max(0, Math.min(100, value)));
	}

	async function handleMerge() {
		if (!mergerState.canMerge) return;

		// Set processing state
		mergerState.setMergeState({ status: 'processing', stage: 'Starting...', percent: 0 });

		const files = mergerState.images.map((img) => img.file);
		const options = {
			direction: mergerState.direction,
			background: mergerState.background,
			overlapSensitivity: mergerState.overlapSensitivity
		};

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

<div class="w-full max-w-3xl mx-auto">
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
			overlapSensitivity={mergerState.overlapSensitivity}
			onBackgroundChange={handleBackgroundChange}
			onDirectionChange={handleDirectionChange}
			onOverlapSensitivityChange={handleOverlapSensitivityChange}
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
</div>

<!-- Error dialog -->
{#if mergerState.hasError && mergerState.mergeState.status === 'error'}
	<ErrorDialog error={mergerState.mergeState.error} onDismiss={handleErrorDismiss} />
{/if}
