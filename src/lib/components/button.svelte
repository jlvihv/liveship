<script lang="ts">
	let { children, className, onClick }: { children: any; className: string; onClick: any } =
		$props();
</script>

<button
	class={`rounded-full p-4 text-lg shadow-lg transition duration-200 hover:shadow-xl focus:scale-95 ${className}`}
	onclick={onClick}
>
	{#if children}
		{@render children()}
	{/if}
</button>
