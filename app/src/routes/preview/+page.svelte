<script lang="ts">
	import Preview from '$lib/components/Preview.svelte';
	import { mergerState } from '$lib/mergerState.svelte';
	import { goto } from '$app/navigation';
	import { resolve } from '$app/paths';

	import { onMount } from 'svelte';

	onMount(async () => {
		// If no result exists, redirect back to home
		if (mergerState.mergeState.status !== 'success') {
			await goto(resolve('/'));
		}
	});

	async function handleReturn() {
		await goto(resolve('/'));
	}
</script>

<div class="container mx-auto p-4 max-w-4xl space-y-6">
	<div class="flex items-center justify-between">
		<button class="btn preset-tonal-primary" onclick={handleReturn}>
			<svg class="w-5 h-5 mr-2" fill="none" stroke="currentColor" viewBox="0 0 24 24">
				<path
					stroke-linecap="round"
					stroke-linejoin="round"
					stroke-width="2"
					d="M10 19l-7-7m0 0l7-7m-7 7h18"
				/>
			</svg>
			Return to Editor
		</button>
		<h1 class="h3">Result Preview</h1>
		<div class="w-[120px]"></div>
		<!-- Spacer for centering -->
	</div>

	{#if mergerState.mergeState.status === 'success'}
		<div class="card p-4 space-y-4 bg-surface-100 dark:bg-surface-800">
			<!-- Re-use existing Preview component but perhaps without the scroll container constraint if possible, 
                 or we can just use it as is since it has the download button logic built-in.
                 Wait, the user wanted a "full page" preview enabling scroll. 
                 The existing Preview component has a max-height of 400px. 
                 We might need to modify Preview.svelte to accept a class or style prop, 
                 or creating a dedicated full page preview here.
                 
                 Let's look at Preview.svelte again. It has internal state for downloading.
                 Ideally we reuse logic.
                 
                 Option 1: Pass a prop to Preview.svelte to disable max-height.
                 Option 2: Extract download logic.
                 
                 Let's go with Option 1 (modify Preview.svelte) as it's cleaner.
                 I'll update Preview.svelte later to support a "full" mode.
            -->
			<Preview
				blob={mergerState.mergeState.blob}
				height={mergerState.mergeState.height}
				url={mergerState.mergeState.url}
				width={mergerState.mergeState.width}
				fullPage={true}
			/>
		</div>
	{/if}
</div>
