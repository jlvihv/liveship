<script>
	import { Toaster } from 'svelte-sonner';

	let { currentPath = $bindable('new') } = $props();
	let list = [
		{ label: '新增录制', icon: 'home', href: 'new' },
		{ label: '录制历史', icon: 'list', href: 'list' },
		{ label: '录制计划', icon: 'calendar', href: 'plan' },
		{ label: '程序设置', icon: 'calendar', href: 'config' }
	];
</script>

<nav
	class="sticky top-0 flex h-screen w-[240px] flex-col justify-between gap-4 border-r border-gray-300 p-4 dark:border-gray-600"
	data-sveltekit-reload
>
	<div class="flex w-full flex-col gap-4 py-4">
		<div class="flex items-center justify-between pb-4">
			<a href="/" class="text-xl font-bold">LiveShip</a>
			<label class="swap swap-rotate">
				<!-- this hidden checkbox controls the state -->
				<input type="checkbox" data-toggle-theme="dark,light" data-act-class="ACTIVECLASS" />

				<!-- sun icon -->
				<svg
					class="swap-on h-5 w-5 fill-current"
					xmlns="http://www.w3.org/2000/svg"
					viewBox="0 0 24 24"
					><path
						d="M5.64,17l-.71.71a1,1,0,0,0,0,1.41,1,1,0,0,0,1.41,0l.71-.71A1,1,0,0,0,5.64,17ZM5,12a1,1,0,0,0-1-1H3a1,1,0,0,0,0,2H4A1,1,0,0,0,5,12Zm7-7a1,1,0,0,0,1-1V3a1,1,0,0,0-2,0V4A1,1,0,0,0,12,5ZM5.64,7.05a1,1,0,0,0,.7.29,1,1,0,0,0,.71-.29,1,1,0,0,0,0-1.41l-.71-.71A1,1,0,0,0,4.93,6.34Zm12,.29a1,1,0,0,0,.7-.29l.71-.71a1,1,0,1,0-1.41-1.41L17,5.64a1,1,0,0,0,0,1.41A1,1,0,0,0,17.66,7.34ZM21,11H20a1,1,0,0,0,0,2h1a1,1,0,0,0,0-2Zm-9,8a1,1,0,0,0-1,1v1a1,1,0,0,0,2,0V20A1,1,0,0,0,12,19ZM18.36,17A1,1,0,0,0,17,18.36l.71.71a1,1,0,0,0,1.41,0,1,1,0,0,0,0-1.41ZM12,6.5A5.5,5.5,0,1,0,17.5,12,5.51,5.51,0,0,0,12,6.5Zm0,9A3.5,3.5,0,1,1,15.5,12,3.5,3.5,0,0,1,12,15.5Z"
					/></svg
				>

				<!-- moon icon -->
				<svg
					class="swap-off h-5 w-5 fill-current"
					xmlns="http://www.w3.org/2000/svg"
					viewBox="0 0 24 24"
					><path
						d="M21.64,13a1,1,0,0,0-1.05-.14,8.05,8.05,0,0,1-3.37.73A8.15,8.15,0,0,1,9.08,5.49a8.59,8.59,0,0,1,.25-2A1,1,0,0,0,8,2.36,10.14,10.14,0,1,0,22,14.05,1,1,0,0,0,21.64,13Zm-9.5,6.69A8.14,8.14,0,0,1,7.08,5.22v.27A10.15,10.15,0,0,0,17.22,15.63a9.79,9.79,0,0,0,2.1-.22A8.11,8.11,0,0,1,12.14,19.73Z"
					/></svg
				>
			</label>
		</div>
		{#each list as item, index}
			<button
				onclick={() => (currentPath = item.href)}
				class="flex items-center justify-start gap-2 rounded-md px-3 py-2 text-sm font-medium text-gray-600 hover:bg-gray-700 hover:text-white dark:text-gray-300 {currentPath ==
				item.href
					? 'bg-gray-700 text-white'
					: ''}"
			>
				{#if index == 0}
					<svg
						class="h-5 w-5"
						xmlns="http://www.w3.org/2000/svg"
						xmlns:xlink="http://www.w3.org/1999/xlink"
						viewBox="0 0 32 32"
						><path
							d="M21 26H4a2 2 0 0 1-2-2V8a2 2 0 0 1 2-2h17a2 2 0 0 1 2 2v4.06l5.42-3.87A1 1 0 0 1 30 9v14a1 1 0 0 1-1.58.81L23 19.94V24a2 2 0 0 1-2 2zM4 8v16h17v-6a1 1 0 0 1 1.58-.81L28 21.06V10.94l-5.42 3.87A1 1 0 0 1 21 14V8z"
							fill="currentColor"
						></path></svg
					>
				{:else if index == 1}
					<svg
						class="h-5 w-5"
						xmlns="http://www.w3.org/2000/svg"
						xmlns:xlink="http://www.w3.org/1999/xlink"
						viewBox="0 0 32 32"
						><path d="M10 6h18v2H10z" fill="currentColor"></path><path
							d="M10 24h18v2H10z"
							fill="currentColor"
						></path><path d="M10 15h18v2H10z" fill="currentColor"></path><path
							d="M4 15h2v2H4z"
							fill="currentColor"
						></path><path d="M4 6h2v2H4z" fill="currentColor"></path><path
							d="M4 24h2v2H4z"
							fill="currentColor"
						></path></svg
					>
				{:else if index == 2}
					<svg
						class="h-5 w-5"
						xmlns="http://www.w3.org/2000/svg"
						xmlns:xlink="http://www.w3.org/1999/xlink"
						viewBox="0 0 32 32"
						><path
							d="M26 30H6a2.002 2.002 0 0 1-2-2V4a2.002 2.002 0 0 1 2-2h20a2.002 2.002 0 0 1 2 2v24a2.002 2.002 0 0 1-2 2zM6 4v24h20V4z"
							fill="currentColor"
						></path><path d="M9 7h11v2H9z" fill="currentColor"></path><path
							d="M9 12h7v2H9z"
							fill="currentColor"
						></path></svg
					>
				{:else if index == 3}
					<svg
						class="h-5 w-5"
						xmlns="http://www.w3.org/2000/svg"
						xmlns:xlink="http://www.w3.org/1999/xlink"
						viewBox="0 0 32 32"
						><path
							d="M27 16.76V16v-.77l1.92-1.68A2 2 0 0 0 29.3 11l-2.36-4a2 2 0 0 0-1.73-1a2 2 0 0 0-.64.1l-2.43.82a11.35 11.35 0 0 0-1.31-.75l-.51-2.52a2 2 0 0 0-2-1.61h-4.68a2 2 0 0 0-2 1.61l-.51 2.52a11.48 11.48 0 0 0-1.32.75l-2.38-.86A2 2 0 0 0 6.79 6a2 2 0 0 0-1.73 1L2.7 11a2 2 0 0 0 .41 2.51L5 15.24v1.53l-1.89 1.68A2 2 0 0 0 2.7 21l2.36 4a2 2 0 0 0 1.73 1a2 2 0 0 0 .64-.1l2.43-.82a11.35 11.35 0 0 0 1.31.75l.51 2.52a2 2 0 0 0 2 1.61h4.72a2 2 0 0 0 2-1.61l.51-2.52a11.48 11.48 0 0 0 1.32-.75l2.42.82a2 2 0 0 0 .64.1a2 2 0 0 0 1.73-1l2.28-4a2 2 0 0 0-.41-2.51zM25.21 24l-3.43-1.16a8.86 8.86 0 0 1-2.71 1.57L18.36 28h-4.72l-.71-3.55a9.36 9.36 0 0 1-2.7-1.57L6.79 24l-2.36-4l2.72-2.4a8.9 8.9 0 0 1 0-3.13L4.43 12l2.36-4l3.43 1.16a8.86 8.86 0 0 1 2.71-1.57L13.64 4h4.72l.71 3.55a9.36 9.36 0 0 1 2.7 1.57L25.21 8l2.36 4l-2.72 2.4a8.9 8.9 0 0 1 0 3.13L27.57 20z"
							fill="currentColor"
						></path><path
							d="M16 22a6 6 0 1 1 6-6a5.94 5.94 0 0 1-6 6zm0-10a3.91 3.91 0 0 0-4 4a3.91 3.91 0 0 0 4 4a3.91 3.91 0 0 0 4-4a3.91 3.91 0 0 0-4-4z"
							fill="currentColor"
						></path></svg
					>
				{/if}
				{item.label}
			</button>
		{/each}
	</div>
	<a
		href="https://github.com/jlvihv/liveship"
		target="_blank"
		class="flex items-center justify-between rounded-md px-3 py-2 text-sm font-medium text-gray-600 hover:bg-gray-700 hover:text-white dark:text-gray-300"
	>
		<span class="flex items-center justify-center gap-2">
			<svg
				class="h-5 w-5"
				xmlns="http://www.w3.org/2000/svg"
				xmlns:xlink="http://www.w3.org/1999/xlink"
				viewBox="0 0 32 32"
				><path
					d="M16 2a14 14 0 0 0-4.43 27.28c.7.13 1-.3 1-.67v-2.38c-3.89.84-4.71-1.88-4.71-1.88a3.71 3.71 0 0 0-1.62-2.05c-1.27-.86.1-.85.1-.85a2.94 2.94 0 0 1 2.14 1.45a3 3 0 0 0 4.08 1.16a2.93 2.93 0 0 1 .88-1.87c-3.1-.36-6.37-1.56-6.37-6.92a5.4 5.4 0 0 1 1.44-3.76a5 5 0 0 1 .14-3.7s1.17-.38 3.85 1.43a13.3 13.3 0 0 1 7 0c2.67-1.81 3.84-1.43 3.84-1.43a5 5 0 0 1 .14 3.7a5.4 5.4 0 0 1 1.44 3.76c0 5.38-3.27 6.56-6.39 6.91a3.33 3.33 0 0 1 .95 2.59v3.84c0 .46.25.81 1 .67A14 14 0 0 0 16 2z"
					fill-rule="evenodd"
					fill="currentColor"
				></path></svg
			>
			Github</span
		>
		<span>
			<svg
				class="h-4 w-4 text-gray-400 dark:text-gray-500"
				xmlns="http://www.w3.org/2000/svg"
				xmlns:xlink="http://www.w3.org/1999/xlink"
				viewBox="0 0 24 24"
				><g fill="none"
					><path
						d="M6.25 4.5A1.75 1.75 0 0 0 4.5 6.25v11.5c0 .966.783 1.75 1.75 1.75h11.5a1.75 1.75 0 0 0 1.75-1.75v-4a.75.75 0 0 1 1.5 0v4A3.25 3.25 0 0 1 17.75 21H6.25A3.25 3.25 0 0 1 3 17.75V6.25A3.25 3.25 0 0 1 6.25 3h4a.75.75 0 0 1 0 1.5h-4zM13 3.75a.75.75 0 0 1 .75-.75h6.5a.75.75 0 0 1 .75.75v6.5a.75.75 0 0 1-1.5 0V5.56l-5.22 5.22a.75.75 0 0 1-1.06-1.06l5.22-5.22h-4.69a.75.75 0 0 1-.75-.75z"
						fill="currentColor"
					></path></g
				></svg
			>
		</span>
	</a>
</nav>

<Toaster />
