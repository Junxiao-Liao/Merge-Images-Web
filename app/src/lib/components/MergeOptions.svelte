<script lang="ts">
	import type { Direction, BackgroundColor } from '$lib/types';

	interface Props {
		direction: Direction;
		background: BackgroundColor;
		onDirectionChange: (direction: Direction) => void;
		onBackgroundChange: (background: BackgroundColor) => void;
	}

	let { direction, background, onDirectionChange, onBackgroundChange }: Props = $props();

	const backgroundPresets: { label: string; color: BackgroundColor }[] = [
		{ label: 'White', color: { r: 255, g: 255, b: 255, a: 255 } },
		{ label: 'Black', color: { r: 0, g: 0, b: 0, a: 255 } },
		{ label: 'Transparent', color: { r: 0, g: 0, b: 0, a: 0 } }
	];

	function isCurrentBackground(preset: BackgroundColor): boolean {
		return (
			preset.r === background.r &&
			preset.g === background.g &&
			preset.b === background.b &&
			preset.a === background.a
		);
	}
</script>

<div class="card p-4 space-y-4 bg-surface-100 dark:bg-surface-800">
	<h3 class="font-medium text-surface-900 dark:text-surface-50">Merge Options</h3>

	<!-- Direction toggle -->
	<div class="space-y-2">
		<span class="text-sm text-surface-600 dark:text-surface-400">Direction</span>
		<div class="flex gap-2" aria-label="Merge direction" role="group">
			<button
				class="flex-1 btn {direction === 'vertical'
					? 'preset-filled-primary-500'
					: 'preset-tonal-surface'}"
				aria-pressed={direction === 'vertical'}
				onclick={() => onDirectionChange('vertical')}
			>
				<svg class="w-4 h-4 mr-2" fill="none" stroke="currentColor" viewBox="0 0 24 24">
					<path
						d="M19 14l-7 7m0 0l-7-7m7 7V3"
						stroke-linecap="round"
						stroke-linejoin="round"
						stroke-width="2"
					/>
				</svg>
				Vertical
			</button>
			<button
				class="flex-1 btn {direction === 'horizontal'
					? 'preset-filled-primary-500'
					: 'preset-tonal-surface'}"
				aria-pressed={direction === 'horizontal'}
				onclick={() => onDirectionChange('horizontal')}
			>
				<svg class="w-4 h-4 mr-2" fill="none" stroke="currentColor" viewBox="0 0 24 24">
					<path
						d="M14 5l7 7m0 0l-7 7m7-7H3"
						stroke-linecap="round"
						stroke-linejoin="round"
						stroke-width="2"
					/>
				</svg>
				Horizontal
			</button>
		</div>
	</div>

	<!-- Background color presets -->
	<div class="space-y-2">
		<span class="text-sm text-surface-600 dark:text-surface-400">Background Color</span>
		<div class="flex gap-2" aria-label="Background color" role="group">
			{#each backgroundPresets as preset (preset.label)}
				<button
					style:background-color="rgba({preset.color.r}, {preset.color.g}, {preset.color.b}, {preset
						.color.a / 255})"
					style={preset.color.a === 0
						? 'background-image: linear-gradient(45deg, #ccc 25%, transparent 25%), linear-gradient(-45deg, #ccc 25%, transparent 25%), linear-gradient(45deg, transparent 75%, #ccc 75%), linear-gradient(-45deg, transparent 75%, #ccc 75%); background-size: 10px 10px; background-position: 0 0, 0 5px, 5px -5px, -5px 0px;'
						: ''}
					class="w-10 h-10 rounded border-2 transition-all {isCurrentBackground(preset.color)
						? 'border-primary-500 ring-2 ring-primary-500/50'
						: 'border-surface-300 dark:border-surface-600'}"
					aria-label={preset.label}
					aria-pressed={isCurrentBackground(preset.color)}
					onclick={() => onBackgroundChange(preset.color)}
					title={preset.label}
				></button>
			{/each}
		</div>
		<p class="text-xs text-surface-500">Used for transparent areas</p>
	</div>
</div>
