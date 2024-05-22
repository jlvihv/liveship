<script lang="ts">
	import { flip } from 'svelte/animate';
	import { RecordingStatus, type ApiResponse, type RecordingHistory } from '$lib/model';
	import { onDestroy, onMount } from 'svelte';
	import { toast } from 'svelte-sonner';
	import dayjs from 'dayjs';
	import relativeTime from 'dayjs/plugin/relativeTime';
	import 'dayjs/locale/zh-cn';
	import Dialog from './dialog.svelte';
	import {
		deleteRecordingHistory,
		getAllRecordingHistory,
		openFolderInFileManager
	} from '$lib/api/history';
	import { stopRecord } from '$lib/api/record';
	import { formatFileSize } from '$lib/utils';
	dayjs.extend(relativeTime);
	dayjs.locale('zh-cn');

	let list: RecordingHistory[] = $state([]);
	let dialogUrl = $state('');
	let intervalId: number | undefined = $state();

	async function getHistory() {
		const resp: ApiResponse = await getAllRecordingHistory();
		if (resp.code == 0) {
			list = resp.data as RecordingHistory[];
		} else {
			toast.error('获取录制历史失败', {
				description: resp.message
			});
		}
	}

	// 删除一条历史记录
	async function fetchDeleteHistory(url: string, startTime: number) {
		const resp: ApiResponse = await deleteRecordingHistory(url, startTime);
		if (resp.code == 0) {
			toast.success('删除成功');
			await getHistory();
		} else {
			toast.error('删除失败', {
				description: resp.message
			});
		}
	}

	// 组件挂载时，获取录制历史，并每隔两秒获取一次
	onMount(async () => {
		// 先获取一次，每隔两秒，再获取一次录制历史
		await getHistory();
		intervalId = setInterval(getHistory, 2000);
	});

	// 组件销毁时，清除定时器
	onDestroy(() => {
		if (intervalId) {
			clearInterval(intervalId);
		}
	});

	async function handleStopRecord(url: string) {
		closeDialog();
		if (url === '') {
			toast.error('url 不能为空');
			return;
		}
		let resp: ApiResponse = await stopRecord(url);
		if (resp.code == 0) {
			toast.success('已经停止录制啦');
			await getHistory();
		} else {
			toast.error('停止录制失败', {
				description: resp.message
			});
		}
	}

	// 在文件管理器中打开文件夹
	async function openFolder(path: string) {
		let resp: ApiResponse = await openFolderInFileManager(path);
		if (resp.code == 0) {
			toast.success('已经在文件管理器中打开了');
		} else {
			toast.error('打开文件夹失败', {
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

	// 计算录制时长，传入开始时间和结束时间，结束时间如果为 0, 则计算到当前时间
	function calcDuration(startTime: number, endTime: number) {
		if (endTime === 0) {
			return dayjs(startTime).fromNow(true);
		}
		return dayjs(startTime).to(dayjs(endTime), true);
	}

	function getPlatformIcon(platformKind: string): string {
		switch (platformKind.toLowerCase()) {
			case 'douyin':
				return 'https://www.douyin.com/favicon.ico';
			case 'huya':
				return 'https://www.huya.com/favicon.ico';
			case 'bilibili':
				return 'https://www.bilibili.com/favicon.ico';
			case 'douyu':
				return 'https://www.douyu.com/favicon.ico';
			case 'kuaishou':
				return 'https://m.kuaishou.com/favicon.ico';
			case 'twitch':
				return 'https://m.twitch.tv/favicon.ico?desktop-redirect=true';
			case 'youtube':
				return 'https://m.youtube.com/static/apple-touch-icon-72x72-precomposed.png';
			default:
				return 'unknown';
		}
	}
</script>

<!-- 原生对话框 -->
<Dialog text="确定停止录制吗？">
	<button class="btn w-24" onclick={() => closeDialog()}>取消</button>
	<button class="btn btn-primary w-24" onclick={() => handleStopRecord(dialogUrl)}>确定</button>
</Dialog>

{#snippet icon(platformKind:string)}
	<img class="h-6 w-6" src={getPlatformIcon(platformKind)} alt={platformKind} />
{/snippet}

{#if list.length > 0}
	<div class="overflow-x-auto">
		<table class="table table-zebra">
			<thead>
				<tr>
					<th class="text-center">录制状态</th>
					<th>平台</th>
					<th>主播</th>
					<th>直播间标题</th>
					<th>录制时长</th>
					<th>文件大小</th>
					<th>文件所在位置</th>
					<th>操作</th>
				</tr>
			</thead>
			<tbody>
				{#each list as row, index}
					<tr>
						<td>
							<p class="text-center">
								<span
									class="tooltip inline-block h-2 w-2 rounded-full {row.status ==
									RecordingStatus.Recording
										? 'blink bg-green-500'
										: 'bg-gray-300 dark:bg-gray-600'}"
									data-tip={row.status == RecordingStatus.Recording ? '录制中' : '已结束'}
								>
								</span>
							</p>
						</td>
						<td>
							{#if row.liveInfo?.platformKind}
								{@render icon(row.liveInfo?.platformKind)}
							{/if}
						</td><td>{row.liveInfo?.anchorName}</td>
						<td
							><a
								href={row.url}
								target="_blank"
								class="text-blue-500 transition duration-200 hover:text-blue-700"
								>{row.liveInfo?.title}</a
							></td
						>
						<td>{calcDuration(row.startTime, row.endTime)}</td>
						<td class="min-w-24">{formatFileSize(row.fileSize)}</td>
						<td>
							{#if !row.deleted}
								<button
									onclick={() => openFolder(row.path)}
									class="tooltip text-left"
									data-tip="点击以在文件管理器中打开"
								>
									{row.path}
								</button>
							{:else}
								<span class="text-gray-300 dark:text-gray-600"> 文件已删除 </span>
							{/if}
						</td>
						<td class="min-w-16 text-sm font-medium">
							{#if row.status === RecordingStatus.Recording}
								<button class="text-red-600" onclick={() => openDialog(row.url)}>停止</button>
							{:else}
								<button
									class="text-red-600"
									onclick={() => fetchDeleteHistory(row.url, row.startTime)}>删除</button
								>
							{/if}
						</td>
					</tr>
				{/each}
			</tbody>
		</table>
	</div>
{:else}
	<div class="flex h-full w-full items-center justify-center">
		<p>录制历史为空，先去新增录制吧</p>
	</div>
{/if}

<style>
	@keyframes blink {
		50% {
			opacity: 0;
		}
	}

	.blink {
		animation: blink 2s step-start infinite;
		background: radial-gradient(
			circle,
			rgba(0, 255, 0, 1) 0%,
			rgba(0, 255, 0, 0.6) 70%,
			rgba(0, 255, 0, 0) 100%
		);
	}
</style>
