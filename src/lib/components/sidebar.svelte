<script lang="ts">
	import { page } from '$app/stores';
	import { t } from '@/translations';

	let list = [
		{ label: $t('addRecord'), icon: 'icon-[fluent--filmstrip-32-regular]', href: '/record' },
		{
			label: $t('recordHistory'),
			icon: 'icon-[fluent--text-bullet-list-20-regular]',
			href: '/record/history'
		},
		{
			label: $t('recordPlan'),
			icon: 'icon-[fluent--task-list-square-rtl-24-regular]',
			href: '/record/plan'
		},
		// { label: $t('streamPush'), icon: 'icon-[fluent--video-28-regular]', href: '/live' },
		// {
		// 	label: $t('streamForward'),
		// 	icon: 'icon-[fluent--square-arrow-forward-32-regular]',
		// 	href: '/live/forward'
		// },
		{ label: $t('settings'), icon: 'icon-[fluent--settings-32-regular]', href: '/config' }
	];
</script>

<nav
	class="sticky top-0 flex h-screen w-[240px] flex-shrink-0 flex-col justify-between gap-4 bg-black1 p-4 text-white1"
>
	<div class="flex w-full flex-col gap-4 py-4">
		<div class="flex items-center justify-between pb-4">
			<a href="/" class="text-xl font-bold">LiveShip</a>
		</div>
		{#each list as item}
			<a
				href={item.href}
				class="flex items-center justify-start gap-2 rounded-xl px-3 py-2 font-medium hover:bg-dark {$page
					.url.pathname === item.href
					? 'bg-dark'
					: 'text-white2'}"
			>
				<span class="{item.icon} h-6 w-6"></span>
				{item.label}
			</a>
		{/each}
	</div>
	<a
		href="https://github.com/jlvihv/liveship"
		target="_blank"
		class="flex items-center justify-between rounded-xl px-3 py-2 font-medium text-white2 hover:bg-dark hover:text-white"
	>
		<span class="flex items-center justify-center gap-2">
			<span class="icon-[icomoon-free--github] h-6 w-6"></span>
			Github
		</span>
		<span class="icon-[fluent--open-24-regular] h-4 w-4"></span>
	</a>
</nav>
