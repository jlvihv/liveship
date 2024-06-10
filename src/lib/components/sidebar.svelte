<script>
	import { t } from '@/translations';
	import { Toaster } from 'svelte-sonner';

	let { currentPath = $bindable('new') } = $props();
	let list = [
		{ label: $t('addRecord'), icon: 'home', href: 'new' },
		{ label: $t('recordHistory'), icon: 'list', href: 'list' },
		{ label: $t('recordPlan'), icon: 'calendar', href: 'plan' },
		{ label: $t('settings'), icon: 'calendar', href: 'config' }
	];
</script>

<nav
	class="bg-black1 sticky top-0 flex h-screen w-[240px] flex-shrink-0 flex-col justify-between gap-4 p-4 text-white1"
	data-sveltekit-reload
>
	<div class="flex w-full flex-col gap-4 py-4">
		<div class="flex items-center justify-between pb-4">
			<a href="/" class="text-xl font-bold">LiveShip</a>
		</div>
		{#each list as item, index}
			<button
				onclick={() => (currentPath = item.href)}
				class="flex items-center justify-start gap-2 rounded-xl px-3 py-2 font-medium hover:bg-dark {currentPath ==
				item.href
					? 'bg-dark'
					: 'text-white2'}"
			>
				{#if index == 0}
					<span class="icon-[fluent--video-28-regular] h-6 w-6"></span>
				{:else if index == 1}
					<span class="icon-[fluent--text-bullet-list-20-regular] h-6 w-6"></span>
				{:else if index == 2}
					<span class="icon-[fluent--task-list-square-rtl-24-regular] h-6 w-6"></span>
				{:else if index == 3}
					<span class="icon-[fluent--settings-32-regular] h-6 w-6"></span>
				{/if}
				{item.label}
			</button>
		{/each}
	</div>
	<a
		href="https://github.com/jlvihv/liveship"
		target="_blank"
		class="text-white2 flex items-center justify-between rounded-xl px-3 py-2 font-medium hover:bg-dark hover:text-white"
	>
		<span class="flex items-center justify-center gap-2">
			<span class="icon-[icomoon-free--github] h-6 w-6"></span>
			Github
		</span>
		<span class="icon-[fluent--open-24-regular] h-4 w-4"></span>
	</a>
</nav>

<Toaster />
