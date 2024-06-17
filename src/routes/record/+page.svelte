<script lang="ts">
	import { toast } from 'svelte-sonner';
	import Dialog from '$lib/components/dialog.svelte';
	import {
		closeDialog,
		openDialog,
		debounce,
		getLiveInfoForPlatform,
		getPlatformIcon
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
		StreamingProtocol,
		type RecordingOption
	} from '$lib/model';
	import { t } from '@/translations';
	import Button from '$lib/components/button.svelte';
	import { scale } from 'svelte/transition';
	import { backOut } from 'svelte/easing';
	import CollapsiblePanel from '@/components/CollapsiblePanel.svelte';
	import { getProxyConfig } from '@/proxy';

	// 关键信息变量

	let inputRef: HTMLInputElement | undefined = $state();
	let url = $state('');
	let liveInfo: LiveInfo | undefined = $state();
	let recordStatus = $state(RecordingStatus.NotRecording);
	let autoRecord = $state(false);
	let streamUrl = $state('');
	let errorMessage = $state('');
	let inPlan = $state(false);
	let useProxy = $state(false);
	let recordingOption: RecordingOption = $state({ useProxy: null });
	let queryHistory: { url: string; anchorName: string; platformKind: PlatformKind }[] = $state([]);
	const tryLinks: string[] = [
		'https://live.douyin.com/790601393533',
		'https://www.huya.com/kpl',
		'https://www.twitch.tv/quantumapprentice',
		'https://www.tiktok.com/@mpl.id.official/live',
		'https://www.xiaohongshu.com/hina/livestream/569260855275038789?timestamp=1718351056826&share_source=&share_source_id=null&source=share_out_of_app&host_id=655924fd000000000802cb9e&xhsshare=WeixinSession&appuid=5ed89960000000000101fdef&apptime=1718351058&share_id=4dc90ecf237d4ef7bd112b5b3d96a425'
	];

	// ui 状态变量
	const stopRecordDialogId = 'stopRecord';
	const downloadFfmpegDialogId = 'downloadFfmpeg';
	let visible = $state(false); // 用来触发过渡动画
	let loading = $state(false);
	let requesting = $state(false); // 表示是否正在请求 live info
	let advancedOptions = $state(false);

	onMount(async () => {
		// 组件挂载时，设置 visible 为 true，以触发过渡动画
		visible = true;

		// 由于这是用户面对的第一个页面，我们可以在这里检测用户有没有安装 ffmpeg，如果没有安装的话，弹出对话框，让用户选择是否自动安装
		// 从 localStorage 中获取 ffmpegDownloading，如果正在下载中，就不用管了
		if (localStorage.getItem('ffmpegDownloading') !== 'true') {
			try {
				await invoke('check_ffmpeg_availability');
				// 如果已经安装了 ffmpeg，什么也不做
			} catch (e) {
				// 没安装 ffmpeg，弹出对话框
				openDialog(downloadFfmpegDialogId);
			}
		}

		// 从 localStorage 中获取 queryHistory
		let history = localStorage.getItem('queryHistory');
		if (history) {
			queryHistory = JSON.parse(history);
		}
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

			// 将 url 加入 queryHistory
			queryHistory = queryHistory.filter((h) => h.url !== url);
			queryHistory.unshift({
				url,
				anchorName: liveInfo.anchorName,
				platformKind: liveInfo.platformKind
			});
			// queryHistory = queryHistory.slice(0, 8);
			localStorage.setItem('queryHistory', JSON.stringify(queryHistory));
		} catch (err) {
			errorMessage = $t('parseError');
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

	async function handleUseProxyChange(event: Event) {
		try {
			await tick();
			if (event.target === null) {
				throw new Error('Event target is null');
			}
			const checked = (event.target as HTMLInputElement).checked;
			if (checked) {
				const proxyConfig = await getProxyConfig();
				if (proxyConfig && recordingOption.useProxy === null) {
					recordingOption.useProxy = proxyConfig.address;
				}
			} else {
				recordingOption.useProxy = null;
			}
		} catch (e) {
			console.error('handle use proxy change error: ', e);
		}
	}

	async function handleAddPlan(enabled: boolean) {
		if (url == '') {
			toast.error($t('pleaseInputLiveAddress'));
			return;
		}
		loading = true;
		try {
			// 先找到当前选中的流
			let stream = liveInfo?.streams.find((s) => s.url === streamUrl);
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
			await invoke('add_plan', { plan });
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
			let plan: RecordingPlan = await invoke('get_plan', { url });
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
		try {
			recordStatus = await invoke('start_record', {
				autoRecord,
				stream,
				liveInfo: liveInfo!,
				option: recordingOption
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
			await invoke('download_ffmpeg');
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

{#snippet icon(platformKind:string)}
	<img class="h-6 w-auto object-cover" src={getPlatformIcon(platformKind)} alt={platformKind} />
{/snippet}

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
		class="mb-8 flex flex-col items-center justify-start"
		transition:scale={{ duration: 300, easing: backOut, start: 0.9 }}
	>
		<div
			class="w-1/2 min-w-[600px] transform-gpu transition-all duration-500 ease-out {liveInfo
				? 'mt-16'
				: 'mt-32'}"
		>
			<label
				class="flex h-14 rounded-full border border-gray1 bg-gray1 px-2 text-gray2 forced-color-adjust-none"
			>
				<input
					bind:this={inputRef}
					oninput={handleinput}
					class="m-0 grow resize-none appearance-none overflow-hidden border-none bg-transparent px-0 py-4 pl-4 placeholder-gray2 outline-none focus:text-white1"
					placeholder={$t('inputPlaceholder')}
				/>
				{#if url}
					<button
						class="tooltip ml-4 mr-2 flex items-center"
						data-tip={$t('clear')}
						onclick={() => {
							inputRef!.value = '';
							url = '';
							liveInfo = undefined;
							errorMessage = '';
							useProxy = false;
							recordingOption.useProxy = null;
						}}
					>
						<span
							class="icon-[fluent--dismiss-circle-28-regular] h-6 w-6 text-gray-500 hover:text-white"
						></span>
					</button>
				{/if}
				<span
					class="absolute inset-x-1 bottom-0 left-[360px] h-px w-[120px] bg-gradient-to-r from-fuchsia-400/0 via-slate-400/60 to-fuchsia-400/0"
				></span>
			</label>
		</div>
		{#if errorMessage || liveInfo || requesting || url}
			<!-- 左边显示标题，或错误消息，右边显示刷新按钮 -->
			<div class="flex w-1/2 min-w-[600px] items-start justify-between pt-12">
				{#if liveInfo?.status === LiveStatus.Live}
					<h1 class="flex w-full items-center gap-8 text-2xl font-bold text-white1">
						{@render icon(liveInfo?.platformKind)}
						<p>
							{liveInfo?.title}
						</p>
					</h1>
				{:else if liveInfo?.status === LiveStatus.NotLive}
					<div>
						<h1 class="text-2xl font-bold text-white1">
							{$t('anchor')}
							{liveInfo.anchorName}
							{$t('notInLive')}
						</h1>
						{#if liveInfo.platformKind == PlatformKind.Huya}
							<div class="pt-12">{$t('forHuyaError')}</div>
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
						class="icon-[fluent--arrow-clockwise-28-regular] h-8 w-8 hover:text-white {requesting
							? 'rotate'
							: ''}"
					></span>
				</button>
			</div>

			<div class="w-1/2 min-w-[600px]">
				{#if requesting}
					<div class="flex h-full w-full items-center justify-center">
						<span class="loading loading-dots loading-md"></span>
					</div>
				{:else if liveInfo}
					<div>
						<div class="w-full text-white1">
							{#if liveInfo.status === LiveStatus.Live}
								<div class="flex items-center gap-8 pt-12">
									<div class="avatar online">
										<div class="h-16 overflow-hidden rounded-full">
											<img
												class="object-cover object-center"
												src={liveInfo.anchorAvatar}
												alt={liveInfo.anchorName}
											/>
										</div>
									</div>
									<div class="flex1 flex h-full flex-col gap-4">
										<b>{liveInfo.anchorName}</b>
										<div class="flex gap-8">
											<p class="text-green-500">{$t('living')}</p>
											{#if recordStatus === RecordingStatus.Recording}
												<span class="text-green-500">{$t('recording')}</span>
											{/if}
											<p>
												{liveInfo.viewerCount ? liveInfo.viewerCount + $t('watching') : ''}
											</p>
										</div>
									</div>
								</div>
								<div class="pt-12">
									<div class="grid grid-cols-2 gap-12">
										<div class="flex items-center">
											<select
												bind:value={streamUrl}
												class="w-full border-none focus:outline-none focus:ring-0"
											>
												{#each liveInfo.streams as item}
													<option value={item.url}
														>{item.protocol + ' ' + $t(item.resolution.toLowerCase())}</option
													>
												{/each}
											</select>
										</div>
										{#if recordStatus === RecordingStatus.NotRecording}
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
													onchange={handleCheckedPlan}
												/>
												{inPlan ? $t('inPlan') : $t('notInPlan')}</label
											>
										{/if}
									</div>
								</div>
								{#if recordStatus === RecordingStatus.NotRecording}
									<CollapsiblePanel isOpen={advancedOptions} className="pt-12">
										<div class="grid grid-cols-2 gap-4 pt-8">
											<label
												for="useProxy"
												class="flex w-full cursor-pointer items-center gap-4 py-4"
											>
												<input
													id="useProxy"
													class="checkbox"
													type="checkbox"
													bind:checked={useProxy}
													onchange={handleUseProxyChange}
												/>
												{$t('useProxy')}</label
											>
											{#if useProxy}
												<label
													class="flex items-center rounded-xl bg-gray1 forced-color-adjust-none"
												>
													<input
														type="text"
														class="m-0 grow resize-none appearance-none overflow-hidden bg-transparent px-0 py-4 pl-4 placeholder-gray2 outline-none focus:text-white1"
														placeholder={$t('useProxyPlaceholder')}
														bind:value={recordingOption.useProxy}
													/>
												</label>
											{/if}
											<!-- <label for="convertToMp4" class="flex w-full cursor-pointer items-center gap-4">
											<input id="convertToMp4" class="checkbox" type="checkbox" />
											录制结束后自动转换为 mp4</label
										>
										<label for="autoSplit" class="flex w-full cursor-pointer items-center gap-4">
											<input id="autoSplit" class="checkbox" type="checkbox" />
											自动切分文件</label
										> -->
										</div>
									</CollapsiblePanel>
								{/if}
								<div class="pt-12">
									<div class="flex justify-center p-4">
										{#if loading}
											<span class="loading loading-dots loading-md"></span>
										{:else if recordStatus === RecordingStatus.Recording}
											<a
												class="w-2/3 rounded-full bg-white1 px-6 py-3 text-lg text-black1 shadow-lg transition duration-200 hover:bg-white hover:shadow-xl focus:scale-95"
												href="/record/history"
											>
												<p
													class="transform text-center transition-transform duration-300 hover:-translate-y-1"
												>
													{$t('gotoRecordHistory')}
												</p>
											</a>
										{:else}
											<Button
												white
												className="w-2/3 transform-gpu transition-all duration-500 ease-out"
												onClick={() => startRecord()}
											>
												<p class="transform transition-transform duration-300 hover:-translate-y-1">
													{$t('startRecord')}
												</p>
											</Button>
										{/if}
									</div>
								</div>
							{:else}
								<div class="flex w-full justify-center pt-12">
									<Button white className="w-2/3 " onClick={() => handleAddPlan(true)}>
										<p class="transform transition-transform duration-300 hover:-translate-y-1">
											{$t('addPlan')}
										</p>
									</Button>
								</div>
								<p class="pt-12 text-xs text-gray-600">{$t('addPlanTips')}</p>
							{/if}
						</div>
					</div>
				{/if}
			</div>
		{:else}
			<div class="container mb-8 mt-16 w-1/2 min-w-[600px] gap-8 overflow-x-clip">
				{#if queryHistory.length <= 5}
					<h3 class="pb-4 font-bold">{$t('tryThese')}</h3>
					<div>
						{#each tryLinks as link}
							<div
								class="transform justify-between transition-transform duration-300 hover:-translate-y-1"
							>
								<button
									class="max-w-full py-2 hover:text-white"
									onclick={() => {
							inputRef!.value = link;
							url = link;
							getLiveInfo();
						}}
								>
									<p class="w-full truncate text-left">
										{link}
									</p>
								</button>
							</div>
						{/each}
					</div>
				{:else}
					<h3 class="pb-4 font-bold">{$t('recentSearch')}</h3>
					{#each queryHistory as history}
						<div
							class="group flex w-full transform items-center justify-between transition-transform duration-300 hover:-translate-y-1"
						>
							<button
								class="max-w-2/3 flex-1 py-2 hover:text-white"
								onclick={() => {
								inputRef!.value = history.url;
								url = history.url;
								getLiveInfo();
							}}
							>
								<p class="w-full truncate text-left">
									{`${history.platformKind && history.anchorName ? $t(history.platformKind) + ' - ' : ''}${history.anchorName}` ||
										history.url}
								</p>
							</button>
							<button
								class="icon-[fluent--dismiss-circle-28-regular] mr-8 h-4 w-4 px-2 text-gray-600 opacity-0 hover:text-white group-hover:opacity-100"
								onclick={() => {
									queryHistory = queryHistory.filter((item) => item.url !== history.url);
									localStorage.setItem('queryHistory', JSON.stringify(queryHistory));
								}}
							></button>
						</div>
					{/each}
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
