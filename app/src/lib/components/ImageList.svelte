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
		images = e.detail.items;
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
