<script lang="ts">
	import '../app.css';
	let { children }: { children?: any } = $props();
</script>

{#if children}
	{@render children()}
{/if}
