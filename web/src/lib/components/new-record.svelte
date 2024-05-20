<script lang="ts">
	import {
		LiveStatus,
		RecordingStatus,
		StreamingProtocol,
		type ApiResponse,
		type LiveInfo,
		type Stream
	} from '$lib/model';
	import { toast } from 'svelte-sonner';
	import Dialog from './dialog.svelte';
	import { debounce, getResolutionName } from '$lib/utils';
	import { getLiveInfo } from '$lib/api/live';
	import { getRecordStatus, startRecord, stopRecord } from '$lib/api/record';
	import { onMount } from 'svelte';
	import { addPlan } from '$lib/api/plan';

	let url = $state('');
	// 用一个变量来表示是否正在请求
	let requesting = $state(false);
	// 用一个变量用来表示图标的旋转
	let isRotating = $state(false);
	// 刷新按钮 setTimeout 的 id
	let refreshTimeout: number | null = null;
	let liveInfo: LiveInfo | null = $state(null);
	let errorMessage = $state('');
	let recordStatus = $state(RecordingStatus.NotRecording);
	let loading = $state(false);
	let dialogUrl = $state('');
	let autoRecord = $state(false);
	let isFirst = $state(false);
	let stream: Stream = $state({
		url: '',
		resolution: '',
		protocol: StreamingProtocol.Flv
	});
	let refreshCount = $state(0);

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
	});

	// 防抖调用 api, 500ms 内只调用一次
	const handleinput = debounce(async () => {
		fetchGetLiveInfo(url);
		refreshCount = 0;
	}, 500);

	async function fetchGetLiveInfo(url: string) {
		if (!url) {
			return;
		}
		// 取消之前的 setTimeout
		if (refreshTimeout) {
			clearTimeout(refreshTimeout);
		}
		refreshCount++;
		isRotating = true;
		requesting = true;
		errorMessage = '';
		liveInfo = null;
		let resp: ApiResponse = await getLiveInfo(url);
		requesting = false;
		if (resp.code == 0) {
			liveInfo = resp.data as LiveInfo;
			if (liveInfo.streams.length > 0) {
				stream = liveInfo.streams[0];
			}
			let recordStatusResponse: ApiResponse = await getRecordStatus(url);
			if (recordStatusResponse.code == 0) {
				recordStatus = recordStatusResponse.data as RecordingStatus;
			}
			console.log(liveInfo);
		} else {
			errorMessage = resp.message;
			console.error(resp.message);
		}
		refreshTimeout = setTimeout(() => {
			isRotating = false;
		}, 1000);
	}

	async function handleAddPlan(url: string) {
		if (url == '') {
			toast.error('请先填写直播地址');
			return;
		}
		loading = true;
		let resp: ApiResponse = await addPlan(url);
		if (resp.code == 0) {
			toast.success('已经添加计划啦');
		} else {
			toast.error('添加计划失败', {
				description: resp.message
			});
		}
		loading = false;
	}

	async function handleStartRecord(url: string) {
		if (url == '') {
			toast.error('请先填写直播地址');
			return;
		}
		loading = true;
		let resp: ApiResponse = await startRecord(
			url,
			autoRecord,
			stream,
			liveInfo!.platformKind,
			liveInfo!.anchorName
		);
		if (resp.code == 0) {
			let recordStatusResponse: ApiResponse = await getRecordStatus(url);
			if (recordStatusResponse.code == 0) {
				recordStatus = recordStatusResponse.data as RecordingStatus;
			}
			toast.success('已经开始录制啦');
		} else {
			toast.error('开始录制失败', {
				description: resp.message
			});
		}
		loading = false;
	}

	async function handleStopRecord(url: string) {
		loading = true;
		closeDialog();
		if (url === '') {
			toast.error('url 不能为空');
			return;
		}
		let resp: ApiResponse = await stopRecord(url);
		if (resp.code == 0) {
			let recordStatusResponse: ApiResponse = await getRecordStatus(url);
			if (recordStatusResponse.code == 0) {
				recordStatus = recordStatusResponse.data as RecordingStatus;
			}
			toast.success('已经停止录制啦');
		} else {
			toast.error('停止录制失败', {
				description: resp.message
			});
		}
	}

	function closeDialog() {
		dialogUrl = '';
		let dialog = document.getElementById('dialog') as HTMLDialogElement;
		dialog?.close();
	}

	function openDialog(url: string) {
		dialogUrl = url;
		let dialog = document.getElementById('dialog') as HTMLDialogElement;
		dialog?.showModal();
	}
</script>

<!-- 原生对话框 -->
<Dialog text="确定停止录制吗？">
	<button onclick={() => closeDialog()}>取消</button>
	<button onclick={() => handleStopRecord(dialogUrl)}>确定</button>
</Dialog>

<!-- 一个输入框，每次改变都调用后端方法 -->
<div class="flex h-screen flex-col items-center justify-start">
	<div class="mt-32 w-1/2">
		<input
			bind:value={url}
			class="input input-bordered input-info w-full"
			placeholder="在这里输入直播间地址"
			oninput={handleinput}
		/>
	</div>
	{#if errorMessage || liveInfo || requesting}
		<div class="mt-4 flex w-1/2 justify-end">
			<button class="btn btn-circle" onclick={() => fetchGetLiveInfo(url)}>
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

	<div class="mt-4 w-1/2">
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
									<p>{liveInfo.viewerCount ? liveInfo.viewerCount + ' 在看' : ''}</p>
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
								<button class="btn btn-error w-full" onclick={() => openDialog(url)}
									>停止录制</button
								>
							{:else}
								<button class="btn btn-primary w-full" onclick={() => handleStartRecord(url)}
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
