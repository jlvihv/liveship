<script lang="ts">
	import { toast } from 'svelte-sonner';
	import Dialog from './dialog.svelte';
	import { closeDialog, openDialog, debounce, getResolutionName } from '$lib/utils';
	import { onMount } from 'svelte';
	import { invoke } from '@tauri-apps/api/core';
	import {
		LiveStatus,
		RecordingStatus,
		StreamingProtocol,
		type LiveInfo,
		type Stream
	} from '$lib/model';
	import { createLiveInfo } from '@/store.svelte';

	const stopRecordDialogId = 'stopRecord';
	let url = $state('');
	// 用一个变量来表示是否正在请求
	let requesting = $state(false);
	// 用一个变量用来表示图标的旋转
	let isRotating = $state(false);
	// 刷新按钮 setTimeout 的 id
	let refreshTimeout: number | undefined = $state();
	let liveInfo: LiveInfo | undefined = $state();
	const storeLiveInfo = createLiveInfo();
	let errorMessage = $state('');
	let recordStatus = $state(RecordingStatus.NotRecording);
	let loading = $state(false);
	let autoRecord = $state(false);
	let isFirst = $state(false);
	let stream: Stream = $state({
		url: '',
		resolution: '',
		protocol: StreamingProtocol.Flv
	});
	let refreshCount = $state(0);

	onMount(() => {
		// 如果可以 storeLiveInfo 有值，则给到 liveInfo
		if (storeLiveInfo) {
			liveInfo = storeLiveInfo.liveInfo;
		}
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
		// 取消之前的 setTimeout
		if (refreshTimeout) {
			clearTimeout(refreshTimeout);
		}
		refreshTimeout = setTimeout(() => {
			isRotating = false;
		}, 2000);
		refreshCount++;
		isRotating = true;
		requesting = true;
		errorMessage = '';
		liveInfo = undefined;
		invoke('live_info', { url: url })
			.then((data) => {
				liveInfo = data as LiveInfo;
				storeLiveInfo.set(liveInfo);
				if (liveInfo.streams.length > 0) {
					stream = liveInfo.streams[0];
				}
				invoke('record_status', { url })
					.then((data) => {
						recordStatus = data as RecordingStatus;
					})
					.catch((err) => {
						toast.error('获取录制状态失败', {
							description: err
						});
					});
			})
			.catch((err) => {
				errorMessage = err;
				console.error(err);
			});
		requesting = false;
	}

	async function handleAddPlan(url: string) {
		if (url == '') {
			toast.error('请先填写直播地址');
			return;
		}
		loading = true;
		invoke('add_plan_with_url', { url })
			.then((data) => {
				toast.success('已经添加计划啦');
			})
			.catch((err) => {
				toast.error('添加计划失败', {
					description: err
				});
			});
		loading = false;
	}

	async function startRecord(url: string) {
		if (url == '') {
			toast.error('请先填写直播地址');
			return;
		}
		loading = true;
		invoke('start_record', {
			autoRecord,
			stream,
			liveInfo: liveInfo!
		})
			.then((data) => {
				recordStatus = data as RecordingStatus;
				toast.success('已经开始录制啦');
			})
			.catch((err) => {
				toast.error('开始录制失败', {
					description: err
				});
			});
		loading = false;
	}

	async function stopRecord(url: string) {
		closeDialog(stopRecordDialogId);
		if (url === '') {
			toast.error('url 不能为空');
			return;
		}
		loading = true;
		invoke('stop_record', { url })
			.then((data) => {
				recordStatus = data as RecordingStatus;
				toast.success('已经停止录制啦');
			})
			.catch((err) => {
				toast.error('停止录制失败', {
					description: err
				});
			});
		loading = false;
	}
</script>

<!-- 原生对话框 -->
<Dialog text="确定停止录制吗？" id={stopRecordDialogId}>
	<button
		onclick={() => {
			closeDialog(stopRecordDialogId);
		}}>取消</button
	>
	<button onclick={() => stopRecord(url)}>确定</button>
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
				placeholder="在这里输入直播间地址"
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
				<p>tips: 目前仅支持抖音直播和虎牙直播</p>
				<p>
					您可访问<a
						class="px-2 text-blue-500 transition duration-200 hover:text-blue-700"
						href="https://live.douyin.com"
						target="_blank">https://live.douyin.com</a
					>和<a
						class="px-2 text-blue-500 transition duration-200 hover:text-blue-700"
						href="https://www.huya.com"
						target="_blank">https://www.huya.com</a
					>
					寻找喜爱的直播间进行录制
				</p>
			</div>
		{/if}
		{#if requesting}
			<div class="mt-4 flex items-center space-x-4">
				<div class="skeleton h-12 w-12 rounded-full"></div>
				<div class="space-y-2">
					<div class="skeleton h-4 w-[250px]"></div>
					<div class="skeleton h-4 w-[200px]"></div>
				</div>
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
										<p class="text-green-500">正在直播</p>
									{:else}
										<p class="text-gray-500">未直播</p>
									{/if}
									<p>
										{liveInfo.viewerCount ? liveInfo.viewerCount + ' 在看' : ''}
									</p>
								</div>
							</div>
						</div>
						<div class="pt-8">
							<div class="grid grid-cols-2 gap-8">
								<select bind:value={stream.url} class="select select-info w-full">
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
									以后自动录制该主播</label
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
									}}>停止录制</button
								>
							{:else}
								<button class="btn btn-primary w-full" onclick={() => startRecord(url)}
									>开始录制</button
								>
							{/if}
						</div>
					{:else}
						<div>
							<div>主播 {liveInfo.anchorName} 当前不在播</div>
							<button class="btn btn-primary mt-8 w-full" onclick={() => handleAddPlan(url)}
								>加入录制计划，主播开播后自动录制</button
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
					>强制忽略错误，加入录制计划并自动重试</button
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
