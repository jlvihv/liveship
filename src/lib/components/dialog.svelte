<script lang="ts">
	let { children, text, id }: { children?: any; text: string; id: string } = $props();
</script>

<dialog {id} class="min-w-[500px] max-w-[800px] rounded-xl p-4 shadow-lg">
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

	dialog[open] {
		animation: var(--animation-slide-in-up) forwards;
	}

	dialog:not([open]) {
		pointer-events: none;
		opacity: 0;
	}
</style>
