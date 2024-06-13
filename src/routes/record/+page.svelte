<script lang="ts">
	import { toast } from 'svelte-sonner';
	import Dialog from '$lib/components/dialog.svelte';
	import {
		closeDialog,
		openDialog,
		debounce,
		getResolutionName,
		getLiveInfoForPlatform
	} from '$lib/utils';
	import { onMount, tick } from 'svelte';
	import { invoke } from '@tauri-apps/api/core';
	import {
		LiveStatus,
		PlatformKind,
		RecordingStatus,
		type LiveInfo,
		type Stream,
		type RecordingPlan,
		StreamingProtocol
	} from '$lib/model';
	import { t } from '@/translations';
	import Button from '$lib/components/button.svelte';
	import { scale } from 'svelte/transition';
	import { backOut } from 'svelte/easing';

	// 关键信息变量

	let url = $state('');
	let liveInfo: LiveInfo | undefined = $state();
	let recordStatus = $state(RecordingStatus.NotRecording);
	let autoRecord = $state(false);
	let streamUrl = $state('');
	let errorMessage = $state('');
	let inPlan = $state(false);

	// ui 状态变量
	const stopRecordDialogId = 'stopRecord';
	const downloadFfmpegDialogId = 'downloadFfmpeg';
	let visible = $state(false); // 用来触发过渡动画
	let loading = $state(false);
	let requesting = $state(false); // 表示是否正在请求 live info

	onMount(() => {
		// 组件挂载时，设置 visible 为 true，以触发过渡动画
		visible = true;

		// 由于这是用户面对的第一个页面，我们可以在这里检测用户有没有安装 ffmpeg，如果没有安装的话，弹出对话框，让用户选择是否自动安装
		// 从 localStorage 中获取 ffmpegDownloading，如果正在下载中，就不用管了
		if (localStorage.getItem('ffmpegDownloading') !== 'true') {
			try {
				invoke('check_ffmpeg_availability');
				// 如果已经安装了 ffmpeg，什么也不做
			} catch (e) {
				// 没安装 ffmpeg，弹出对话框
				openDialog(downloadFfmpegDialogId);
			}
		}
	});

	$effect(() => {
		// 当 inPlan 变化时，打印
		console.log('inPlan', inPlan);
	});

	// 防抖调用 api, 500ms 内只调用一次
	const handleinput = debounce(async (event) => {
		url = event?.target.value.trim();
		getLiveInfo();
	}, 500);

	async function getLiveInfo() {
		if (!url) {
			return;
		}

		// 先清空之前的数据
		liveInfo = undefined;
		errorMessage = '';

		// ui 状态设置
		requesting = true;

		try {
			liveInfo = await getLiveInfoForPlatform(url);
			recordStatus = await invoke('record_status', { url });
			await isInPlan();

			if (liveInfo.streams.length > 0) {
				streamUrl = liveInfo.streams[0].url;
			}
		} catch (err) {
			errorMessage = err as string;
		}
		requesting = false;
	}

	async function handleCheckedPlan(event: Event) {
		try {
			await tick();
			if (event.target === null) {
				throw new Error('Event target is null');
			}
			await handleAddPlan((event.target as HTMLInputElement).checked);
		} catch (e) {
			console.error('handle checked plan error: ', e);
		}
	}

	async function handleAddPlan(enabled: boolean) {
		if (url == '') {
			toast.error($t('pleaseInputLiveAddress'));
			return;
		}
		console.log('handle add plan, enabled: ', enabled);
		loading = true;
		try {
			// 先找到当前选中的流
			let stream = liveInfo?.streams.find((s) => s.url === streamUrl);
			console.log('stream', stream);
			let plan: RecordingPlan = {
				url,
				enabled,
				strategy: 'AnchorLive',
				streamProtocol: stream?.protocol || StreamingProtocol.Flv,
				streamResolution: stream?.resolution || '',
				liveInfo: liveInfo!,
				createdAt: new Date().getTime(),
				updatedAt: new Date().getTime()
			};
			invoke('add_plan', { plan });
			if (enabled) {
				toast.success($t('planEnabled'));
			} else {
				toast.success($t('planDisabled'));
			}
		} catch (e) {
			toast.error($t('planAddFailed'), {
				description: e as string
			});
		}
		loading = false;
	}

	// 获取对应 url 的计划信息
	async function getPlan(): Promise<RecordingPlan | null> {
		try {
			let data = await invoke('get_plan', { url });
			let plan = data as RecordingPlan;
			return plan;
		} catch (e) {
			return null;
		}
	}

	// 判断是否在计划中，且计划为开启
	async function isInPlan() {
		const plan = await getPlan();
		inPlan = plan !== null && plan.enabled;
	}

	async function startRecord() {
		if (url == '') {
			toast.error($t('pleaseInputLiveAddress'));
			return;
		}
		loading = true;
		// 从 liveInfo 中获取 stream
		let stream: Stream | undefined = liveInfo?.streams.find((s) => s.url === streamUrl);
		if (!stream) {
			toast.error($t('pleaseSelectStream'));
			loading = false;
			return;
		}
		console.log('stream', stream, liveInfo);
		try {
			recordStatus = await invoke('start_record', {
				autoRecord,
				stream,
				liveInfo: liveInfo!
			});
			toast.success($t('recordAlreadyStarted'));
			await isInPlan();
		} catch (e) {
			toast.error($t('recordStartFailed'), {
				description: e as string
			});
		}
		loading = false;
	}

	async function stopRecord() {
		closeDialog(stopRecordDialogId);
		if (url === '') {
			toast.error($t('urlCannotBeEmpty'));
			return;
		}
		loading = true;
		try {
			recordStatus = await invoke('stop_record', { url });
			toast.success($t('recordAlreadyStopped'));
			await isInPlan();
		} catch (e) {
			toast.error($t('recordStopFailed'), {
				description: e as string
			});
		}
		loading = false;
	}

	async function ffmpegAutoDownload() {
		closeDialog(downloadFfmpegDialogId);
		toast.info($t('downloading'));
		// 写入 localStorage，表示正在下载中
		localStorage.setItem('ffmpegDownloading', 'true');
		try {
			invoke('download_ffmpeg');
			toast.success($t('downloadedFFmpeg'));
		} catch (e) {
			toast.error($t('downloadFFmpegFailed'), {
				description: e as string
			});
		}
		// 最后，删除 localStorage 中的 ffmpegDownloading
		localStorage.removeItem('ffmpegDownloading');
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
	<button autofocus class="btn w-24 rounded-xl" onclick={() => stopRecord()}>{$t('confirm')}</button
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

{#if visible}
	<div
		class="flex h-screen flex-col items-center justify-start"
		transition:scale={{ duration: 300, easing: backOut, start: 0.9 }}
	>
		<div
			class="w-1/2 min-w-[600px] transform-gpu transition-all duration-500 ease-out {liveInfo
				? 'mt-16'
				: 'mt-32'}"
		>
			<label class="flex h-14 rounded-full bg-gray1 px-2 text-gray2 forced-color-adjust-none">
				<input
					oninput={handleinput}
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
		{#if errorMessage || liveInfo || requesting || url}
			<!-- 左边显示标题，或错误消息，右边显示刷新按钮 -->
			<div class="flex w-1/2 min-w-[600px] items-start justify-between pt-8">
				{#if liveInfo?.status === LiveStatus.Live}
					<h1 class="text-2xl font-bold text-white1">
						{liveInfo?.title}
						{#if recordStatus === RecordingStatus.Recording}
							<span class="px-4 text-sm font-normal text-green-500">{$t('recording')}</span>
						{/if}
					</h1>
				{:else if liveInfo?.status === LiveStatus.NotLive}
					<div>
						<h1 class="text-2xl font-bold text-white1">
							{$t('anchor')}
							{liveInfo.anchorName}
							{$t('notInLive')}
						</h1>
						{#if liveInfo.platformKind == PlatformKind.Huya}
							<div>{$t('forHuyaError')}</div>
						{/if}
					</div>
				{:else if errorMessage}
					<div class="text-red-500">{errorMessage}</div>
				{:else}
					<!-- 空标签占位 -->
					<div></div>
				{/if}

				<button class="px-3" onclick={() => getLiveInfo()}>
					<span
						class="icon-[fluent--arrow-clockwise-28-regular] h-8 w-8 {requesting ? 'rotate' : ''}"
					></span>
				</button>
			</div>

			<div class="w-1/2 min-w-[600px]">
				{#if requesting}
					<div class="flex h-full w-full items-center justify-center">
						<span class="loading loading-dots loading-md"></span>
					</div>
				{/if}
				{#if liveInfo}
					<div>
						<div class="w-full text-white1">
							{#if liveInfo.status === LiveStatus.Live}
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
											<p class="text-green-500">{$t('living')}</p>
											<p>
												{liveInfo.viewerCount ? liveInfo.viewerCount + $t('watching') : ''}
											</p>
										</div>
									</div>
								</div>
								<div class="pt-8">
									<div class="grid grid-cols-2 gap-8">
										<select
											bind:value={streamUrl}
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
											<label class="flex w-full cursor-pointer items-center gap-4">
												<input
													class="toggle"
													type="checkbox"
													bind:checked={inPlan}
													onclick={handleCheckedPlan}
												/>
												{inPlan ? $t('inPlan') : $t('notInPlan')}</label
											>
										{/if}
									</div>
								</div>
								<div class="pt-8">
									<div class="flex justify-center p-4">
										{#if loading}
											<span class="loading loading-dots loading-md"></span>
										{:else if recordStatus === RecordingStatus.Recording}
											<a
												class="rounded-full bg-white1 px-6 py-3 text-lg text-black1 shadow-lg transition duration-200 hover:bg-white hover:shadow-xl focus:scale-95"
												href="/record/history">去录制历史中查看</a
											>
										{:else}
											<Button white className=" w-2/3 " onClick={() => startRecord()}
												>{$t('startRecord')}</Button
											>
										{/if}
									</div>
								</div>
							{:else}
								<div class="flex w-full justify-center pt-8">
									<Button white className="w-2/3 " onClick={() => handleAddPlan(true)}
										>{$t('addPlan')}</Button
									>
								</div>
							{/if}
						</div>
					</div>
				{/if}
			</div>
		{/if}
	</div>
{/if}

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
