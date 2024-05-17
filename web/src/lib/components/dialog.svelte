<script lang="ts">
	let { children, text } = $props();
</script>

<dialog id="dialog" class="w-96 rounded-lg p-4 shadow-lg dark:shadow-gray-700">
	<h2 class="p-4 text-lg">{text}</h2>
	<menu class="mt-16 flex justify-end gap-4 p-4">
		{#if children}
			{@render children()}
		{/if}
	</menu>
</dialog>

<style>
	dialog::backdrop {
		background-color: rgba(0, 0, 0, 0.5);
		backdrop-filter: blur(6px);
	}
</style>
