<script lang="ts">
	import '../app.css';
	import favicon from '$lib/assets/favicon.svg';

	let { children } = $props();
</script>

<svelte:head>
	<link href={favicon} rel="icon" />
	<title>MergeImages</title>
</svelte:head>

<div class="min-h-screen flex flex-col bg-surface-50 dark:bg-surface-900">
	<!-- Header -->
	<header
		class="p-4 bg-surface-100 dark:bg-surface-800 border-b border-surface-200 dark:border-surface-700"
	>
		<div class="container mx-auto flex items-center justify-between max-w-5xl">
			<h1 class="text-xl font-bold text-surface-900 dark:text-surface-50">MergeImages</h1>
			<span class="text-sm text-surface-500">Client-side only</span>
		</div>
	</header>

	<!-- Main content -->
	<main class="flex-1 container mx-auto p-4 max-w-5xl">
		{@render children()}
	</main>

	<!-- Footer -->
	<footer
		class="p-4 text-center text-sm text-surface-500 border-t border-surface-200 dark:border-surface-700"
	>
		No uploads - All processing happens in your browser
	</footer>
</div>
