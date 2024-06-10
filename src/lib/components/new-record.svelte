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
		type Stream,
		type RecordingPlan
	} from '$lib/model';
	import { t } from '@/translations';
	import Button from './button.svelte';

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
	let inPlan = $state(false);
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
			})
			.finally(async () => {
				inPlan = await isInPlan(url);
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

	// 获取对应 url 的计划信息
	async function getPlan(url: string): Promise<RecordingPlan | null> {
		try {
			let data = await invoke('get_plan', { url });
			let plan = data as RecordingPlan;
			return plan;
		} catch (e) {
			return null;
		}
	}

	// 判断是否在计划中，且计划为开启
	async function isInPlan(url: string): Promise<boolean> {
		const plan = await getPlan(url);
		return plan !== null && plan.enabled;
	}

	async function addOrDeletePlan(event: any) {
		const isChecked = event.target.checked;
		if (!isChecked) {
			deletePlan(url);
		} else {
			addPlan(url);
		}
	}

	async function deletePlan(url: string) {
		invoke('delete_plan', { url })
			.then(() => {
				toast.success($t('planDeleted'));
			})
			.catch((e) => {
				toast.error($t('planDeleteFailed'), {
					description: e
				});
			})
			.finally(async () => {
				inPlan = await isInPlan(url);
			});
	}

	async function addPlan(url: string) {
		invoke('add_plan_with_url', { url })
			.then(() => {
				toast.success($t('planAdded'));
			})
			.catch((e) => {
				toast.error($t('planAddFailed'), {
					description: e
				});
			})
			.finally(async () => {
				inPlan = await isInPlan(url);
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
		class="btn w-24 rounded-xl"
		onclick={() => {
			closeDialog(stopRecordDialogId);
		}}>{$t('cancel')}</button
	>
	<!-- svelte-ignore a11y_autofocus -->
	<button autofocus class="btn w-24 rounded-xl" onclick={() => stopRecord(url)}
		>{$t('confirm')}</button
	>
</Dialog>

<!-- 询问是否自动安装 ffmpeg -->
<Dialog text={$t('confirmInstallFFmpeg')} id={downloadFfmpegDialogId}>
	<button
		class="btn w-24 rounded-xl"
		onclick={() => {
			closeDialog(downloadFfmpegDialogId);
		}}>{$t('no')}</button
	>
	<button class="btn w-32 rounded-xl" onclick={() => ffmpegAutoDownload()}
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
		<label class="flex h-14 rounded-full bg-gray1 px-2 text-gray2 forced-color-adjust-none">
			<input
				oninput={handleinput}
				bind:value={url}
				class="m-0 grow resize-none appearance-none overflow-hidden border-none bg-transparent px-0 py-4 pl-4 placeholder-gray2 outline-none focus:text-white1"
				placeholder={$t('inputPlaceholder')}
			/>
			{#if url}
				<button
					class="tooltip mr-2 flex items-center"
					data-tip={$t('clear')}
					onclick={() => {
						url = '';
						liveInfo = undefined;
						errorMessage = '';
					}}
				>
					<span class="icon-[fluent--dismiss-circle-28-regular] h-6 w-6 text-gray-500"></span>
				</button>
			{/if}
		</label>
	</div>
	{#if errorMessage || liveInfo || requesting}
		<div class="mt-4 flex w-1/2 min-w-[600px] justify-end">
			<button onclick={() => getLiveInfo(url)}>
				<span class="icon-[fluent--arrow-clockwise-28-regular] h-8 w-8 {isRotating ? 'rotate' : ''}"
				></span>
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
				<div class="w-full text-white1">
					{#if liveInfo.status !== LiveStatus.NotLive}
						<div>
							<h1 class="text-2xl font-bold">
								{liveInfo.title}
								{#if recordStatus === RecordingStatus.Recording}
									<span class="px-4 text-sm font-normal text-green-500">{$t('recording')}</span>
								{/if}
							</h1>
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
								<select
									bind:value={stream_url}
									class="w-full border-none focus:outline-none focus:ring-0"
								>
									{#each liveInfo.streams as item}
										<option value={item.url}
											>{item.protocol + ' ' + getResolutionName(item.resolution)}</option
										>
									{/each}
								</select>
								{#if recordStatus !== RecordingStatus.Recording}
									<!-- 可选框，以后自动录制该主播 -->
									<label for="autoRecord" class="flex w-full cursor-pointer items-center gap-4">
										<input
											class="checkbox"
											type="checkbox"
											id="autoRecord"
											bind:checked={autoRecord}
										/>
										{$t('autoRecord')}</label
									>
								{:else}
									<!-- 判断是否在计划中，可通过此可选框加入或取消计划 -->
									<label for="autoRecord" class="flex w-full cursor-pointer items-center gap-4">
										<input
											class="checkbox"
											type="checkbox"
											bind:checked={inPlan}
											onchange={(event) => addOrDeletePlan(event)}
										/>
										{inPlan ? $t('inPlan') : $t('notInPlan')}</label
									>
								{/if}
							</div>
						</div>
						<div class="pt-8">
							{#if loading}
								<div class="mt-2 flex w-full items-center justify-center">
									<span class="loading loading-dots loading-md"></span>
								</div>
							{:else if recordStatus === RecordingStatus.Recording}
								<div class="flex justify-center p-4">
									<Button
										className="text-white w-2/3 bg-red-800 hover:bg-red-700"
										onClick={() => {
											openDialog(stopRecordDialogId);
										}}
										>{$t('stopRecord')}
									</Button>
								</div>
							{:else}
								<div class="flex justify-center p-4">
									<Button
										className="text-black1 w-2/3 bg-white1 hover:bg-white"
										onClick={() => startRecord(url)}>{$t('startRecord')}</Button
									>
								</div>
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
