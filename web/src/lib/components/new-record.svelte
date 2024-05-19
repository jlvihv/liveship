<script lang="ts">
	import { LiveStatus, RecordingStatus, type ApiResponse, type LiveInfo } from '$lib/model';
	import { toast } from 'svelte-sonner';
	import Dialog from './dialog.svelte';
	import { debounce, getResolutionName } from '$lib/utils';
	import { getLiveInfo } from '$lib/api/live';
	import { getRecordStatus, startRecord, stopRecord } from '$lib/api/record';
	import { onMount } from 'svelte';

	let url = $state('');
	// 用一个变量来表示是否正在请求
	let requesting = $state(false);
	let liveInfo: LiveInfo | null = $state(null);
	let errorMessage = $state('');
	let storagePath = $state('');
	let storageFilename = $state('');
	let streamKind = $state('hls');
	let streamResolution: string = $state('HD1');
	let recordStatus = $state(RecordingStatus.NotRecording);
	let loading = $state(false);
	let dialogUrl = $state('');
	let autoRecord = $state(false);
	let isFirst = $state(false);

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
		if (!url) {
			return;
		}
		requesting = true;
		errorMessage = '';
		liveInfo = null;
		storageFilename = '';
		let resp: ApiResponse = await getLiveInfo(url);
		requesting = false;
		if (resp.code == 0) {
			liveInfo = resp.data as LiveInfo;
			let recordStatusResponse: ApiResponse = await getRecordStatus(url);
			if (recordStatusResponse.code == 0) {
				recordStatus = recordStatusResponse.data as RecordingStatus;
			}
		} else {
			errorMessage = resp.message;
		}
	}, 500);

	async function handleStartRecord(url: string) {
		if (url == '') {
			toast.error('请先填写直播地址');
			return;
		}
		if (streamResolution == '') {
			toast.error('请先选择分辨率');
			return;
		}
		loading = true;
		let resp: ApiResponse = await startRecord(
			url,
			storagePath,
			storageFilename,
			streamKind,
			streamResolution,
			autoRecord
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
		loading = false;
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
			placeholder="在这里输入抖音直播间地址"
			oninput={handleinput}
		/>
	</div>
	<div class="mt-4 w-1/2">
		{#if isFirst}
			<div class="py-4 text-sm text-gray-500">
				<p>tips: 目前仅支持抖音直播</p>
				<p>
					您可访问<a
						class="px-2 text-blue-500 transition duration-200 hover:text-blue-700"
						href="https://live.douyin.com"
						target="_blank">https://live.douyin.com</a
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
						<div class="pt-8">
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
							<div class="grid grid-cols-2">
								<select bind:value={streamResolution} class="w-full">
									{#each liveInfo.streams as item}
										<option value={item.url}>{getResolutionName(item.resolution)}</option>
									{/each}
								</select>
							</div>
						</div>
						<div class="pt-8">
							<!-- 可选框，以后自动录制该主播 -->
							<div class="flex gap-4 pb-8">
								<label for="autoRecord">
									<input type="checkbox" id="autoRecord" bind:checked={autoRecord} />
									以后自动录制该主播</label
								>
							</div>
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
						</div>
					{/if}
				</div>
			</div>
		{/if}
		{#if errorMessage}
			<div class="mt-4 text-red-500">{errorMessage}</div>
		{/if}
	</div>
</div>

<style>
</style>
