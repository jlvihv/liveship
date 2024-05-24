<script lang="ts">
	let { children, text, id }: { children?: any; text: string; id: string } = $props();
</script>

<dialog {id} class="w-[500px] rounded-lg p-4 shadow-lg dark:shadow-gray-700/70">
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
