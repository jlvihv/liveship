<script lang="ts">
	import { fade, scale } from 'svelte/transition';
	import type { AppConfig } from '$lib/model';
	import { onMount } from 'svelte';
	import { toast } from 'svelte-sonner';
	import { invoke } from '@tauri-apps/api/core';
	import { t } from '@/translations';
	import { backOut } from 'svelte/easing';
	import Button from '@/components/button.svelte';
	import { platform } from '@tauri-apps/plugin-os';

	let config: AppConfig | undefined = $state();
	let rawConfig: AppConfig | undefined = $state();
	let changed = $derived.by(() => {
		return JSON.stringify(config) !== JSON.stringify(rawConfig);
	});
	let ffmpegVersion = $state('');
	let timeoutId: number | undefined = $state();
	let language = $state('en');

	onMount(async () => {
		// 获取保存在 localStorage 中的语言设置
		language = localStorage.getItem('lang') || 'en';
		await getConfig();

		const os = await platform();
		localStorage.setItem('os', os);
	});

	// 改变语言，并保存到 localStorage
	async function changeLanguage(lang: string) {
		localStorage.setItem('lang', lang);
		location.reload();
	}

	// 当 config.ffmepgPath 改变时，清空 ffmpegVersion
	function handleFfmpegPathChange() {
		ffmpegVersion = '';
	}

	function getFFmpegPathPlaceholder() {
		const os = localStorage.getItem('os');
		if (os == 'windows') {
			return ' e.g.: C:\\ffmpeg\\bin\\ffmpeg.exe';
		} else {
			return ' e.g.: /usr/local/bin/ffmpeg';
		}
	}

	async function getConfig() {
		try {
			rawConfig = await invoke('get_config');
			config = JSON.parse(JSON.stringify(rawConfig));
		} catch (e) {
			toast.error($t('getSettingsFailed'), {
				description: e as string
			});
		}
	}

	async function setConfig() {
		try {
			await invoke('set_config', { config });
			toast.success($t('saveSuccess'));
			rawConfig = JSON.parse(JSON.stringify(config));
		} catch (e) {
			toast.error($t('saveFailed'), {
				description: e as string
			});
		}
	}

	async function checkFFmpeg(path: string) {
		// 清除之前的定时器
		if (timeoutId) {
			clearTimeout(timeoutId);
		}
		try {
			ffmpegVersion = await invoke('check_ffmpeg_version', { path });
			toast.success($t('ffmpegPathAvailable'));
			timeoutId = setTimeout(() => {
				ffmpegVersion = '';
			}, 10000);
		} catch (e) {
			toast.error($t('ffmpegPathUnavailable'), {
				description: e as string
			});
		}
	}
</script>

{#if config}
	<div
		class="grid gap-8 p-8 text-white2"
		transition:scale={{ duration: 300, easing: backOut, start: 0.9 }}
	>
		<label class="flex items-center gap-8">
			{$t('language')}
			<select
				bind:value={language}
				class="min-w-32 border-none focus:outline-none focus:ring-0"
				onchange={() => changeLanguage(language)}
			>
				<option value="en">English</option>
				<option value="cn">中文</option>
			</select>
		</label>
		<label class="flex h-14 items-center rounded-xl bg-gray1 forced-color-adjust-none">
			<span class="flex h-full items-center border-r-2 border-dark px-4">
				{$t('ffmpegPath')}
			</span>
			<input
				type="text"
				class="m-0 grow resize-none appearance-none overflow-hidden bg-transparent px-0 py-4 pl-4 placeholder-gray2 outline-none focus:text-white1"
				placeholder={$t('ffmpegPathPlaceholder') + getFFmpegPathPlaceholder()}
				bind:value={config.ffmpegPath}
				oninput={handleFfmpegPathChange}
			/>
			{#if ffmpegVersion}
				<span class="text-green-500" transition:fade>{ffmpegVersion}</span>
			{/if}
			<button class="px-4" onclick={() => {
					checkFFmpeg(config!.ffmpegPath);
				}}>{$t('check')}</button>
		</label>

		<label class="flex h-14 items-center rounded-xl bg-gray1 forced-color-adjust-none">
			<span class="flex h-full items-center border-r-2 border-dark px-4">
				{$t('savePath')}
			</span>
			<input
				type="text"
				class="m-0 grow resize-none appearance-none overflow-hidden bg-transparent px-0 py-4 pl-4 placeholder-gray2 outline-none focus:text-white1"
				placeholder={$t('savePathPlaceholder')}
				bind:value={config.savePath}
			/>
		</label>

		<!-- <label class="input input-bordered flex items-center gap-8">
			{$t('pollInterval')}
			<input
				type="number"
				class="grow text-gray-700 dark:text-gray-200"
				placeholder=""
				bind:value={config.liveInfoCheckInterval}
			/>
		</label> -->

		<div class="flex items-center justify-center gap-8 p-8">
			{#if changed}
				<Button
					white
					className="w-36"
					onClick={() => (config = JSON.parse(JSON.stringify(rawConfig)))}>{$t('cancel')}</Button
				>
				<Button white className="w-36" onClick={async () => await setConfig()}>{$t('save')}</Button>
			{/if}
		</div>
	</div>
{:else}
	<div class="flex h-full w-full items-center justify-center">
		<span class="loading loading-dots loading-md"></span>
	</div>
{/if}
