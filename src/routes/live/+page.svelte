<script lang="ts">
	import { invoke } from '@tauri-apps/api/core';
	import { open } from '@tauri-apps/plugin-dialog';

	import { generateFFmpegCommand, getImageInfo, type MediaInfo, type Effects } from '$lib/ffmpeg';
	import { scale } from 'svelte/transition';
	import { backOut } from 'svelte/easing';
	import { onMount } from 'svelte';

	const xiaohongshuPushServer = 'rtmp://live-push.xhscdn.com/live';
	let key = $state('');
	let inputFile = $state('');
	let visible = $state(false);

	onMount(() => {
		visible = true;
	});
	async function push() {
		try {
			let info = await getImageInfo(inputFile);
			console.log('info: ', info);
			let url = `${xiaohongshuPushServer}/${key.trim()}`;
			let ffmpegCommand = generateFFmpegCommand(info, url);
			console.log('开始推流 ffmpegCommand: ', ffmpegCommand);
			await invoke('execute_ffmpeg_command', { ffmpegCommand });
		} catch (error) {
			console.error('execute ffmpeg command error: ', error);
		}
	}

	async function filePicker() {
		const selected = await open({
			multiple: false,
			directory: false,
			filter: {
				name: 'Images',
				extensions: ['jpg', 'jpeg', 'png']
			}
		});
		console.log('selected: ', selected);
		if (selected?.path) {
			inputFile = selected.path;
		}
	}
</script>

{#if visible}
	<div
		class="flex h-full items-center justify-center"
		transition:scale={{ duration: 300, easing: backOut, start: 0.9 }}
	>
		<div class="grid w-2/3 gap-16">
			<p>小红书推流</p>
			<button class="file-input" onclick={() => filePicker()}>选择文件</button>
			<label class="flex h-14 rounded-full bg-gray1 px-2 text-gray2 forced-color-adjust-none">
				<input
					bind:value={key}
					class="m-0 grow resize-none appearance-none overflow-hidden border-none bg-transparent px-0 py-4 pl-4 placeholder-gray2 outline-none focus:text-white1"
					placeholder="请输入推流密钥"
				/>
			</label>
			{#if key && inputFile}
				<button class="btn btn-primary" onclick={() => push()}> 开始推流 </button>
			{/if}
		</div>
	</div>
{/if}
