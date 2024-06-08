<script lang="ts">
	import { toast } from 'svelte-sonner';
	import Dialog from './dialog.svelte';
	import {
		closeDialog,
		openDialog,
		debounce,
		getResolutionName,
		getLiveInfoForPlatform
	} from '$lib/utils';
	import { onMount } from 'svelte';
	import { invoke } from '@tauri-apps/api/core';
	import {
		LiveStatus,
		PlatformKind,
		RecordingStatus,
		type LiveInfo,
		type Stream
	} from '$lib/model';
	import { t } from '@/translations';

	let url = $state('');
	// 用一个变量来表示是否正在请求
	let requesting = $state(false);
	// 用一个变量用来表示图标的旋转
	let isRotating = $state(false);
	// 刷新按钮 setTimeout 的 id
	let refreshTimeout: number | undefined = $state();
	let liveInfo: LiveInfo | undefined = $state();
	let errorMessage = $state('');
	let recordStatus = $state(RecordingStatus.NotRecording);
	let loading = $state(false);
	let autoRecord = $state(false);
	let isFirst = $state(false);
	let stream_url = $state('');
	let refreshCount = $state(0);
	const stopRecordDialogId = 'stopRecord';
	const downloadFfmpegDialogId = 'downloadFfmpeg';

	onMount(() => {
		// 从 localStorage 中获取 isFrist
		let first = localStorage.getItem('isFirst');
		isFirst = first === null;
		// 1 分钟后设置 isFirst 为 false
		if (isFirst) {
			setTimeout(() => {
				isFirst = false;
				localStorage.setItem('isFirst', 'false');
			}, 60000);
		}

		// 由于这是用户面对的第一个页面，我们可以在这里检测用户有没有安装 ffmpeg，如果没有安装的话，弹出对话框，让用户选择是否自动安装
		// 从 localStorage 中获取 ffmpegDownloading，如果正在下载中，就不用管了
		let downloading = localStorage.getItem('ffmpegDownloading');
		if (downloading !== 'true') {
			invoke('check_ffmpeg_availability')
				.then((_data) => {
					// 如果已经安装了 ffmpeg，就不用管了
				})
				.catch((err) => {
					openDialog(downloadFfmpegDialogId);
				});
		}
	});

	// 防抖调用 api, 500ms 内只调用一次
	const handleinput = debounce(async () => {
		getLiveInfo(url);
		refreshCount = 0;
	}, 500);

	async function getLiveInfo(url: string) {
		if (!url) {
			return;
		}
		// 先清空之前的数据
		liveInfo = undefined;
		requesting = true;
		isRotating = true;
		// 取消之前的 setTimeout
		if (refreshTimeout) {
			clearTimeout(refreshTimeout);
		}
		refreshTimeout = setTimeout(() => {
			isRotating = false;
		}, 2000);
		refreshCount++;
		errorMessage = '';
		try {
			liveInfo = await getLiveInfoForPlatform(url);
		} catch (err) {
			requesting = false;
			isRotating = false;
			errorMessage = err as string;
			return;
		}
		if (liveInfo.streams.length > 0) {
			stream_url = liveInfo.streams[0].url;
		}
		invoke('record_status', { url })
			.then((data) => {
				recordStatus = data as RecordingStatus;
			})
			.catch((err) => {
				toast.error($t('getRecordStatusFailed'), {
					description: err
				});
			});
		requesting = false;
		isRotating = false;
	}

	async function handleAddPlan(url: string) {
		if (url == '') {
			toast.error($t('pleaseInputLiveAddress'));
			return;
		}
		loading = true;
		invoke('add_plan_with_url', { url })
			.then((data) => {
				toast.success($t('planAlreadyAdded'));
			})
			.catch((err) => {
				toast.error($t('planAddFailed'), {
					description: err
				});
			})
			.finally(() => {
				loading = false;
			});
	}

	async function startRecord(url: string) {
		if (url == '') {
			toast.error($t('pleaseInputLiveAddress'));
			return;
		}
		loading = true;
		// 从 liveInfo 中获取 stream
		let stream: Stream | undefined = liveInfo?.streams.find((s) => s.url === stream_url);
		if (!stream) {
			toast.error($t('pleaseSelectStream'));
			loading = false;
			return;
		}
		console.log('stream', stream, liveInfo);
		invoke('start_record', {
			autoRecord,
			stream,
			liveInfo: liveInfo!
		})
			.then((data) => {
				recordStatus = data as RecordingStatus;
				toast.success($t('recordAlreadyStarted'));
			})
			.catch((err) => {
				toast.error($t('recordStartFailed'), {
					description: err
				});
			})
			.finally(() => {
				loading = false;
			});
	}

	async function stopRecord(url: string) {
		closeDialog(stopRecordDialogId);
		if (url === '') {
			toast.error($t('urlCannotBeEmpty'));
			return;
		}
		loading = true;
		invoke('stop_record', { url })
			.then((data) => {
				recordStatus = data as RecordingStatus;
				toast.success($t('recordAlreadyStopped'));
			})
			.catch((err) => {
				toast.error($t('recordStopFailed'), {
					description: err
				});
			})
			.finally(() => {
				loading = false;
			});
	}

	async function ffmpegAutoDownload() {
		closeDialog(downloadFfmpegDialogId);
		toast.info($t('downloading'));
		// 写入 localStorage，表示正在下载中
		localStorage.setItem('ffmpegDownloading', 'true');
		invoke('download_ffmpeg')
			.then((data) => {
				toast.success($t('downloadedFFmpeg'));
			})
			.catch((err) => {
				toast.error($t('downloadFFmpegFailed'), {
					description: err
				});
			})
			.finally(() => {
				// 下载完成后，删除 localStorage 中的 ffmpegDownloading
				localStorage.removeItem('ffmpegDownloading');
			});
	}
</script>

<!-- 原生对话框 -->
<Dialog text={$t('confirmStopRecord')} id={stopRecordDialogId}>
	<button
		class="btn w-24"
		onclick={() => {
			closeDialog(stopRecordDialogId);
		}}>{$t('cancel')}</button
	>
	<button class="btn btn-primary w-24" onclick={() => stopRecord(url)}>{$t('confirm')}</button>
</Dialog>

<!-- 询问是否自动安装 ffmpeg -->
<Dialog text={$t('confirmInstallFFmpeg')} id={downloadFfmpegDialogId}>
	<button
		class="btn w-24"
		onclick={() => {
			closeDialog(downloadFfmpegDialogId);
		}}>{$t('no')}</button
	>
	<button class="btn btn-primary w-24" onclick={() => ffmpegAutoDownload()}
		>{$t('autoInstall')}</button
	>
</Dialog>

<!-- 一个输入框，每次改变都调用后端方法 -->
<div class="flex h-screen flex-col items-center justify-start">
	<div
		class="w-1/2 min-w-[600px] transform-gpu transition-all duration-500 ease-out {liveInfo
			? 'mt-16'
			: 'mt-32'}"
	>
		<label class="input input-bordered input-info flex w-full items-center">
			<input
				class="grow"
				bind:value={url}
				placeholder={$t('inputPlaceholder')}
				oninput={handleinput}
			/>
			{#if url}
				<button
					onclick={() => {
						url = '';
						liveInfo = undefined;
						errorMessage = '';
					}}
				>
					<svg
						class="h-6 w-6 text-gray-300 dark:text-gray-600"
						xmlns="http://www.w3.org/2000/svg"
						xmlns:xlink="http://www.w3.org/1999/xlink"
						viewBox="0 0 24 24"
						><g fill="none"
							><path
								d="M12 2c5.523 0 10 4.477 10 10s-4.477 10-10 10S2 17.523 2 12S6.477 2 12 2zm0 1.5a8.5 8.5 0 1 0 0 17a8.5 8.5 0 0 0 0-17zm3.446 4.897l.084.073a.75.75 0 0 1 .073.976l-.073.084L13.061 12l2.47 2.47a.75.75 0 0 1 .072.976l-.073.084a.75.75 0 0 1-.976.073l-.084-.073L12 13.061l-2.47 2.47a.75.75 0 0 1-.976.072l-.084-.073a.75.75 0 0 1-.073-.976l.073-.084L10.939 12l-2.47-2.47a.75.75 0 0 1-.072-.976l.073-.084a.75.75 0 0 1 .976-.073l.084.073L12 10.939l2.47-2.47a.75.75 0 0 1 .976-.072z"
								fill="currentColor"
							></path></g
						></svg
					>
				</button>
			{/if}
		</label>
	</div>
	{#if errorMessage || liveInfo || requesting}
		<div class="mt-4 flex w-1/2 min-w-[600px] justify-end">
			<button onclick={() => getLiveInfo(url)}>
				<svg
					xmlns="http://www.w3.org/2000/svg"
					xmlns:xlink="http://www.w3.org/1999/xlink"
					viewBox="0 0 24 24"
					class="h-8 w-8 {isRotating ? 'rotate' : ''}"
					><path
						d="M17.65 6.35a7.95 7.95 0 0 0-6.48-2.31c-3.67.37-6.69 3.35-7.1 7.02C3.52 15.91 7.27 20 12 20a7.98 7.98 0 0 0 7.21-4.56c.32-.67-.16-1.44-.9-1.44c-.37 0-.72.2-.88.53a5.994 5.994 0 0 1-6.8 3.31c-2.22-.49-4.01-2.3-4.48-4.52A6.002 6.002 0 0 1 12 6c1.66 0 3.14.69 4.22 1.78l-1.51 1.51c-.63.63-.19 1.71.7 1.71H19c.55 0 1-.45 1-1V6.41c0-.89-1.08-1.34-1.71-.71l-.64.65z"
						fill="currentColor"
					></path></svg
				>
			</button>
		</div>
	{/if}

	<div class="mt-4 w-1/2 min-w-[600px]">
		{#if isFirst}
			<div class="py-4 text-sm text-gray-500">
				<p>{$t('tips')}</p>
			</div>
		{/if}
		{#if requesting}
			<div class="flex h-full w-full items-center justify-center">
				<span class="loading loading-dots loading-md"></span>
			</div>
		{/if}
		{#if liveInfo}
			<div>
				<div class="w-full">
					{#if liveInfo.status !== LiveStatus.NotLive}
						<div>
							<h1 class="text-2xl font-bold">{liveInfo.title}</h1>
						</div>

						<div class="flex items-center gap-8 pt-8">
							<div class="avatar online">
								<div class="h-16 overflow-hidden rounded-full">
									<img
										class="object-cover object-center"
										src={liveInfo.anchorAvatar}
										alt={liveInfo.anchorName}
									/>
								</div>
							</div>
							<div class="flex1 flex h-full flex-col">
								<b>{liveInfo.anchorName}</b>
								<div class="flex gap-8">
									{#if liveInfo.status === LiveStatus.Live}
										<p class="text-green-500">{$t('living')}</p>
									{:else}
										<p class="text-gray-500">{$t('notLiving')}</p>
									{/if}
									<p>
										{liveInfo.viewerCount ? liveInfo.viewerCount + $t('watching') : ''}
									</p>
								</div>
							</div>
						</div>
						<div class="pt-8">
							<div class="grid grid-cols-2 gap-8">
								<select bind:value={stream_url} class="select select-info w-full">
									{#each liveInfo.streams as item}
										<option value={item.url}
											>{item.protocol + ' ' + getResolutionName(item.resolution)}</option
										>
									{/each}
								</select>
								<!-- 可选框，以后自动录制该主播 -->
								<label for="autoRecord" class="flex w-full items-center gap-4">
									<input
										class="checkbox"
										type="checkbox"
										id="autoRecord"
										bind:checked={autoRecord}
									/>
									{$t('autoRecord')}</label
								>
							</div>
						</div>
						<div class="pt-8">
							{#if loading}
								<div class="mt-2 flex w-full items-center justify-center">
									<span class="loading loading-dots loading-md"></span>
								</div>
							{:else if recordStatus === RecordingStatus.Recording}
								<button
									class="btn btn-error w-full"
									onclick={() => {
										openDialog(stopRecordDialogId);
									}}>{$t('stopRecord')}</button
								>
							{:else}
								<button class="btn btn-primary w-full" onclick={() => startRecord(url)}
									>{$t('startRecord')}</button
								>
							{/if}
						</div>
					{:else}
						<div>
							<div>{$t('anchor')} {liveInfo.anchorName} {$t('notInLive')}</div>
							{#if liveInfo.platformKind == PlatformKind.Huya}
								<div>{$t('forHuyaError')}</div>
							{/if}
							<button class="btn btn-primary mt-8 w-full" onclick={() => handleAddPlan(url)}
								>{$t('addPlan')}</button
							>
						</div>
					{/if}
				</div>
			</div>
		{/if}
		{#if errorMessage}
			<div class="text-red-500">{errorMessage}</div>
			{#if refreshCount >= 5}
				<button class="btn btn-primary mt-8 w-full" onclick={() => handleAddPlan(url)}
					>{$t('forceIgnoreError')}</button
				>
			{/if}
		{/if}
	</div>
</div>

<style>
	.rotate {
		animation: rotation 1s linear;
	}

	@keyframes rotation {
		from {
			transform: rotate(0deg);
		}
		to {
			transform: rotate(360deg);
		}
	}
</style>
