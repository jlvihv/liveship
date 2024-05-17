<script lang="ts">
	import { fade } from 'svelte/transition';
	import type { ApiResponse, AppConfig } from '$lib/model';
	import { onMount, tick } from 'svelte';
	import { toast } from 'svelte-sonner';
	import { getConfig, setConfig } from '$lib/api/config';
	import { checkFFmpeg } from '$lib/api/ffmpeg';

	let config: AppConfig | undefined = $state();
	let rawConfig: AppConfig | undefined = $state();
	let changed = $derived.by(() => {
		return JSON.stringify(config) !== JSON.stringify(rawConfig);
	});
	let ffmpegVersion = $state('');
	let timeoutId: number | undefined = $state();

	// 当 config.ffmepgPath 改变时，清空 ffmpegVersion
	function handleFfmpegPathChange() {
		ffmpegVersion = '';
	}

	async function getAppConfig() {
		let resp: ApiResponse = await getConfig();
		if (resp.code == 0) {
			rawConfig = resp.data as AppConfig;
			// config 是一个深度克隆版本，与 rawConfig 使用不同的引用
			config = JSON.parse(JSON.stringify(rawConfig));
		} else {
			toast.error('获取配置失败', {
				description: resp.message
			});
		}
	}

	onMount(async () => {
		await getAppConfig();
	});

	async function saveConfig() {
		let resp: ApiResponse = await setConfig(config!);
		if (resp.code == 0) {
			toast.success('保存成功');
			rawConfig = JSON.parse(JSON.stringify(config));
		} else {
			toast.error('保存失败', {
				description: resp.message
			});
		}
	}

	async function fetchCheckFFmpeg(path: string) {
		// 清除之前的定时器
		if (timeoutId) {
			clearTimeout(timeoutId);
		}
		let resp: ApiResponse = await checkFFmpeg(path);
		if (resp.code == 0) {
			// 以空格分隔，取第 3 个元素
			let versionString = resp.data as string;
			ffmpegVersion = versionString.split(' ')[2];
			toast.success('FFmpeg 路径可用');
			// 10 秒钟后或路径改变后清空版本信息
			timeoutId = setTimeout(() => {
				ffmpegVersion = '';
			}, 10000);
		} else {
			toast.error('FFmpeg 路径不可用', {
				description: resp.message
			});
		}
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
					fetchCheckFFmpeg(config!.ffmpegPath);
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
				<button class="btn btn-wide btn-primary" onclick={async () => await saveConfig()}
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
