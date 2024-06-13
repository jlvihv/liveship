<script lang="ts">
	import '../app.css';
	import { Toaster } from 'svelte-sonner';
	import Sidebar from '$lib/components/sidebar.svelte';

	let { children }: { children?: any } = $props();
	let currentPath = $state('new');
</script>

<div class="flex">
	<Sidebar />

	<div class="w-full overflow-hidden">
		{#if children}
			{@render children()}
		{/if}
	</div>
</div>

<Toaster />
