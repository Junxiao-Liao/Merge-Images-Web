<script lang="ts">
	import type { ImageFile } from '$lib/types';

	interface Props {
		image: ImageFile;
		index: number;
		total: number;
		onMoveUp: () => void;
		onMoveDown: () => void;
		onRemove: () => void;
	}

	let { image, index, total, onMoveUp, onMoveDown, onRemove }: Props = $props();
</script>

<div class="card p-2 relative group bg-surface-100 dark:bg-surface-800">
	<img
		class="w-full aspect-square object-cover rounded"
		alt={image.name}
		src={image.thumbnailUrl}
	/>

	<p class="text-xs truncate mt-1 text-surface-600 dark:text-surface-300">{image.name}</p>

	<!-- Position indicator -->
	<span
		class="absolute bottom-8 left-2 bg-primary-500 text-white text-xs px-2 py-0.5 rounded font-medium"
	>
		{index + 1}
	</span>

	<!-- Control buttons (visible on hover) -->
	<div
		class="absolute top-1 right-1 flex gap-1 opacity-0 group-hover:opacity-100 focus-within:opacity-100 transition-opacity"
	>
		<button
			class="btn btn-sm preset-filled-surface-200-800 p-1 min-w-0"
			aria-label="Move up"
			disabled={index === 0}
			onclick={onMoveUp}
			title="Move up"
		>
			<svg class="w-4 h-4" fill="none" stroke="currentColor" viewBox="0 0 24 24">
				<path d="M5 15l7-7 7 7" stroke-linecap="round" stroke-linejoin="round" stroke-width="2" />
			</svg>
		</button>
		<button
			class="btn btn-sm preset-filled-surface-200-800 p-1 min-w-0"
			aria-label="Move down"
			disabled={index === total - 1}
			onclick={onMoveDown}
			title="Move down"
		>
			<svg class="w-4 h-4" fill="none" stroke="currentColor" viewBox="0 0 24 24">
				<path d="M19 9l-7 7-7-7" stroke-linecap="round" stroke-linejoin="round" stroke-width="2" />
			</svg>
		</button>
		<button
			class="btn btn-sm preset-filled-error-500 p-1 min-w-0"
			aria-label="Remove"
			onclick={onRemove}
			title="Remove"
		>
			<svg class="w-4 h-4" fill="none" stroke="currentColor" viewBox="0 0 24 24">
				<path
					d="M6 18L18 6M6 6l12 12"
					stroke-linecap="round"
					stroke-linejoin="round"
					stroke-width="2"
				/>
			</svg>
		</button>
	</div>
</div>
