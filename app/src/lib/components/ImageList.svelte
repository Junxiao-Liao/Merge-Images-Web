<script lang="ts">
	import { dndzone } from 'svelte-dnd-action';
	import ImageItem from './ImageItem.svelte';
	import type { ImageFile } from '$lib/types';

	interface Props {
		images: ImageFile[];
		onReorder: (reorderedImages: ImageFile[]) => void;
		onRemove: (imageId: string) => void;
	}

	let { images, onReorder, onRemove }: Props = $props();

	const flipDurationMs = 200;

	function handleDndConsider(e: CustomEvent<{ items: ImageFile[] }>) {
		// Just update visual state immediately for drag feel,
		// but don't persist yet or we conflict with parent prop updates
		// Actually, standard dndzone pattern in Svelte 5 with runes:
		// We might need to just call reorder?
		// But usually we need to update local variable to avoid flicker.
		// However `images` is a prop. In Svelte 5 we cannot reassign props directly if they are not $bindable.
		// Since we didn't make `images` bindable in parent, we should likely just rely on parent updates
		// OR we should have a local state.

		// For now, let's just emit event. If it causes flicker we can improve.
		// Actually dndzone requires updating the items array it is watching immediately.
		// Since `images` is a prop, we can't mutate it.
		// We should probably rely on the parent (ImageMerger) to handle the temporary state
		// or make it bindable.
		// But ImageMerger is using a global store now.

		// Let's call onReorder even for consider.
		onReorder(e.detail.items);
	}

	function handleDndFinalize(e: CustomEvent<{ items: ImageFile[] }>) {
		onReorder(e.detail.items);
	}

	function moveImage(fromIndex: number, toIndex: number) {
		if (toIndex < 0 || toIndex >= images.length) return;

		const newImages = [...images];
		const [moved] = newImages.splice(fromIndex, 1);
		newImages.splice(toIndex, 0, moved);
		onReorder(newImages);
	}
</script>

<div
	class="grid grid-cols-2 sm:grid-cols-3 md:grid-cols-4 gap-4"
	onconsider={handleDndConsider}
	onfinalize={handleDndFinalize}
	use:dndzone={{ items: images, flipDurationMs, type: 'images' }}
>
	{#each images as image, index (image.id)}
		<ImageItem
			{image}
			{index}
			onMoveDown={() => moveImage(index, index + 1)}
			onMoveUp={() => moveImage(index, index - 1)}
			onRemove={() => onRemove(image.id)}
			total={images.length}
		/>
	{/each}
</div>
