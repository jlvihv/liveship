<script lang="ts">
	import { fade } from 'svelte/transition';
	import type { AppConfig } from '$lib/model';
	import { onMount, tick } from 'svelte';
	import { toast } from 'svelte-sonner';
	import { invoke } from '@tauri-apps/api/core';

	let config: AppConfig | undefined = $state();
	let rawConfig: AppConfig | undefined = $state();
	let changed = $derived.by(() => {
		return JSON.stringify(config) !== JSON.stringify(rawConfig);
	});
	let ffmpegVersion = $state('');
	let timeoutId: number | undefined = $state();

	onMount(async () => {
		await getConfig();
	});

	// 当 config.ffmepgPath 改变时，清空 ffmpegVersion
	function handleFfmpegPathChange() {
		ffmpegVersion = '';
	}

	async function getConfig() {
		invoke('get_config')
			.then((data) => {
				rawConfig = data as AppConfig;
				config = JSON.parse(JSON.stringify(rawConfig));
			})
			.catch((e) =>
				toast.error('获取配置失败', {
					description: e
				})
			);
	}

	async function setConfig() {
		invoke('set_config', { config })
			.then(() => {
				toast.success('保存成功');
				rawConfig = JSON.parse(JSON.stringify(config));
			})
			.catch((e) =>
				toast.error('保存失败', {
					description: e
				})
			);
	}

	async function checkFFmpeg(path: string) {
		// 清除之前的定时器
		if (timeoutId) {
			clearTimeout(timeoutId);
		}
		invoke('check_ffmpeg', { path })
			.then((data) => {
				// 以空格分割，取第三个元素
				let arr = (data as string).split(' ');
				if (arr.length < 3) {
					ffmpegVersion = '';
				} else {
					ffmpegVersion = arr[2];
				}
				toast.success('FFmpeg 路径可用');
				timeoutId = setTimeout(() => {
					ffmpegVersion = '';
				}, 10000);
			})
			.catch((err) => {
				toast.error('FFmpeg 路径不可用', {
					description: err
				});
			});
	}
</script>

{#if config}
	<div class="grid gap-8 p-8 text-gray-400">
		<label class="input input-bordered flex items-center gap-8">
			ffmpeg 路径
			<input
				type="text"
				class="grow text-gray-700 dark:text-gray-200"
				placeholder=""
				bind:value={config.ffmpegPath}
				oninput={handleFfmpegPathChange}
			/>
			{#if ffmpegVersion}
				<span class="text-green-500" transition:fade>{ffmpegVersion}</span>
			{/if}
			<button onclick={() => {
					checkFFmpeg(config!.ffmpegPath);
				}}>检查</button>
		</label>

		<label class="input input-bordered flex items-center gap-8">
			文件保存路径
			<input
				type="text"
				class="grow text-gray-700 dark:text-gray-200"
				placeholder=""
				bind:value={config.savePath}
			/>
		</label>

		<label class="input input-bordered flex items-center gap-8">
			开播信息轮询间隔 (单位秒)
			<input
				type="number"
				class="grow text-gray-700 dark:text-gray-200"
				placeholder=""
				bind:value={config.liveInfoCheckInterval}
			/>
		</label>

		<div class="flex items-center justify-center gap-8 p-8">
			{#if changed}
				<button
					class="btn btn-wide dark:bg-gray-400 dark:text-gray-800"
					onclick={() => (config = JSON.parse(JSON.stringify(rawConfig)))}>取消</button
				>
				<button class="btn btn-primary btn-wide" onclick={async () => await setConfig()}
					>保存</button
				>
			{/if}
		</div>
	</div>
{:else}
	<div class="flex h-full w-full items-center justify-center">
		<span class="loading loading-dots loading-md"></span>
	</div>
{/if}
