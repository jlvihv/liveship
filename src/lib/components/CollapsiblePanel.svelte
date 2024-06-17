<script lang="ts">
	import { t } from '@/translations';
	import { backOut } from 'svelte/easing';
	import { scale } from 'svelte/transition';

	let {
		children,
		className,
		isOpen = $bindable(false)
	}: { children?: any; className?: string; isOpen: boolean } = $props();

	function togglePanel() {
		isOpen = !isOpen;
	}
</script>

<div class={`${className}`}>
	<!-- svelte-ignore a11y_no_static_element_interactions -->
	<!-- svelte-ignore a11y_click_events_have_key_events -->
	<div class="flex h-6 cursor-pointer items-center gap-2" onclick={togglePanel}>
		<span
			class="icon-[fluent--chevron-right-28-regular] h-full w-5 font-bold transition-transform"
			class:rotate-90={isOpen}
		></span>
		<h2 class="flex h-full flex-1 items-center text-lg">{$t('advancedOptions')}</h2>
	</div>
	{#if isOpen}
		<div transition:scale={{ duration: 300, easing: backOut, start: 0.9 }}>
			{#if children}
				{@render children()}
			{/if}
		</div>
	{/if}
</div>

<style>
	.rotate-90 {
		transform: rotate(90deg);
	}
	.transition-transform {
		transition: transform 0.3s;
	}
</style>
