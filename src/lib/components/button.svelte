<script lang="ts">
	let {
		children,
		className,
		onClick,
		black,
		white,
		red
	}: {
		children: any;
		className: string;
		onClick: any;
		black?: boolean;
		white?: boolean;
		red?: boolean;
	} = $props();
</script>

<button
	class={`rounded-full px-6 py-3 text-lg shadow-lg transition duration-200 hover:shadow-xl focus:scale-95 ${className} ${
		black ? 'bg-black1 text-white1 hover:bg-black' : ''
	} ${white ? 'bg-white1 text-black1 hover:bg-white' : ''} ${red ? 'bg-red-800 text-white1 hover:bg-red-700' : ''}`}
	onclick={onClick}
>
	{#if children}
		{@render children()}
	{/if}
</button>
