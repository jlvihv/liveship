<script lang="ts">
	import '../app.css';
	import { Toaster } from 'svelte-sonner';
	let { children }: { children?: any } = $props();
</script>

{#if children}
	{@render children()}
{/if}

<Toaster />
