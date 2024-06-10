<script lang="ts">
	import { fade } from 'svelte/transition';
	import type { AppConfig } from '$lib/model';
	import { onMount } from 'svelte';
	import { toast } from 'svelte-sonner';
	import { invoke } from '@tauri-apps/api/core';
	import { t } from '@/translations';

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

	async function getConfig() {
		invoke('get_config')
			.then((data) => {
				rawConfig = data as AppConfig;
				config = JSON.parse(JSON.stringify(rawConfig));
			})
			.catch((e) =>
				toast.error($t('getSettingsFailed'), {
					description: e
				})
			);
	}

	async function setConfig() {
		invoke('set_config', { config })
			.then(() => {
				toast.success($t('saveSuccess'));
				rawConfig = JSON.parse(JSON.stringify(config));
			})
			.catch((e) =>
				toast.error($t('saveFailed'), {
					description: e
				})
			);
	}

	async function checkFFmpeg(path: string) {
		// 清除之前的定时器
		if (timeoutId) {
			clearTimeout(timeoutId);
		}
		invoke('check_ffmpeg_version', { path })
			.then((data) => {
				ffmpegVersion = data as string;
				toast.success($t('ffmpegPathAvailable'));
				timeoutId = setTimeout(() => {
					ffmpegVersion = '';
				}, 10000);
			})
			.catch((err) => {
				toast.error($t('ffmpegPathUnavailable'), {
					description: err
				});
			});
	}
</script>

{#if config}
	<div class="text-white2 grid gap-8 p-8">
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
				placeholder={$t('ffmpegPathPlaceholder')}
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
				placeholder=""
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
				<button
					class="btn btn-wide"
					onclick={() => (config = JSON.parse(JSON.stringify(rawConfig)))}>{$t('cancel')}</button
				>
				<button class="btn btn-primary btn-wide" onclick={async () => await setConfig()}
					>{$t('save')}</button
				>
			{/if}
		</div>
	</div>
{:else}
	<div class="flex h-full w-full items-center justify-center">
		<span class="loading loading-dots loading-md"></span>
	</div>
{/if}
