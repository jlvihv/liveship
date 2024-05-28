<script lang="ts">
	// icon fluent 28

	import { RecordingStatus, type RecordingHistory } from '$lib/model';
	import { onDestroy, onMount } from 'svelte';
	import { toast } from 'svelte-sonner';
	import dayjs from 'dayjs';
	import relativeTime from 'dayjs/plugin/relativeTime';
	import 'dayjs/locale/zh-cn';
	import Dialog from './dialog.svelte';
	import { closeDialog, openDialog, formatFileSize, getPlatformIcon } from '$lib/utils';
	import { invoke } from '@tauri-apps/api/core';
	dayjs.extend(relativeTime);
	dayjs.locale('zh-cn');

	let stopRecordDialogId = 'stopRecord';
	let deleteHistoryDialogId = 'deleteHistory';
	let list: RecordingHistory[] = $state([]);
	let dialogUrl = $state('');
	let intervalId: number | undefined = $state();
	let deleteHistoryParams: { url: string; startTime: number; deleted: boolean } | undefined =
		$state();

	// 组件挂载时，获取录制历史，并每隔两秒获取一次
	onMount(async () => {
		// 先获取一次，每隔两秒，再获取一次录制历史
		await getAllHistory();
		intervalId = setInterval(getAllHistory, 2000);
	});

	// 组件销毁时，清除定时器
	onDestroy(() => {
		if (intervalId) {
			clearInterval(intervalId);
		}
	});

	async function getAllHistory() {
		invoke('get_all_history')
			.then((data) => {
				list = data as RecordingHistory[];
			})
			.catch((e) => {
				toast.error('获取录制历史失败', {
					description: e
				});
			});
	}

	// 删除一条历史记录
	async function deleteHistory(url: string, startTime: number, deleteFile: boolean) {
		closeDialog(deleteHistoryDialogId);
		deleteHistoryParams = undefined;
		invoke('delete_history', { url, startTime, deleteFile })
			.then(() => {
				toast.success('删除成功');
				getAllHistory();
			})
			.catch((e) => {
				toast.error('删除失败', {
					description: e
				});
			});
	}

	async function stopRecord(url: string) {
		closeDialog(stopRecordDialogId);
		dialogUrl = '';
		if (url === '') {
			toast.error('url 不能为空');
			return;
		}
		invoke('stop_record', { url })
			.then(() => {
				toast.success('已经停止录制啦');
				getAllHistory();
			})
			.catch((e) => {
				toast.error('停止录制失败', {
					description: e
				});
			});
	}

	// 在文件管理器中打开文件夹
	async function openInFolder(path: string) {
		invoke('open_in_folder', { path })
			.then(() => {
				toast.success('已经在文件管理器中打开了');
			})
			.catch((e) => {
				toast.error('打开文件夹失败', {
					description: e
				});
			});
	}

	// 计算录制时长，传入开始时间和结束时间，结束时间如果为 0, 则计算到当前时间
	function calcDuration(startTime: number, endTime: number) {
		if (endTime === 0) {
			return dayjs(startTime).fromNow(true);
		}
		return dayjs(startTime).to(dayjs(endTime), true);
	}
</script>

<!-- 原生对话框 -->
<Dialog id={stopRecordDialogId} text="确定停止录制吗？">
	<button
		class="btn w-24"
		onclick={() => {
			closeDialog(stopRecordDialogId);
			dialogUrl = '';
		}}>取消</button
	>
	<button class="btn btn-primary w-32" onclick={() => stopRecord(dialogUrl)}>确定</button>
</Dialog>

<Dialog
	id={deleteHistoryDialogId}
	text={deleteHistoryParams?.deleted ? '确定删除此记录吗？' : '仅删除记录，还是同时删除文件？'}
>
	<button
		class="btn w-24"
		onclick={() => {
			closeDialog(deleteHistoryDialogId);
			deleteHistoryParams = undefined;
		}}>取消</button
	>
	<button
		class="btn btn-primary w-32"
		onclick={() =>
			deleteHistory(
				deleteHistoryParams!.url,
				deleteHistoryParams!.startTime,
				false
			)}
		>{deleteHistoryParams?.deleted ? '确定' : '仅删除记录'}</button
	>
	{#if !deleteHistoryParams?.deleted}
		<button
			class="btn btn-primary w-32"
			onclick={() => deleteHistory(
			  deleteHistoryParams!.url,
    deleteHistoryParams!.startTime,
    true
			)}
			>同时删除文件</button
		>
	{/if}
</Dialog>

{#snippet icon(platformKind:string)}
	<img class="h-6 w-6" src={getPlatformIcon(platformKind)} alt={platformKind} />
{/snippet}

{#if list.length > 0}
	<div class="overflow-auto">
		<table class="table table-zebra">
			<thead>
				<tr>
					<th class="text-center">状态</th>
					<th>平台</th>
					<th>主播</th>
					<th>直播间</th>
					<th>时长</th>
					<th>大小</th>
					<th>文件</th>
					<th class="min-w-20">操作</th>
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
									onclick={() => openInFolder(row.path)}
									class="tooltip text-left"
									data-tip="在文件管理器中打开"
								>
									<!-- {row.path} -->

									<svg
										class="h-6 w-6 hover:text-blue-500"
										xmlns="http://www.w3.org/2000/svg"
										xmlns:xlink="http://www.w3.org/1999/xlink"
										viewBox="0 0 28 28"
										><g fill="none"
											><path
												d="M10.207 4c.46 0 .908.141 1.284.402l.156.12L14.022 6.5h9.728a2.25 2.25 0 0 1 2.229 1.938l.016.158l.005.154v13a2.25 2.25 0 0 1-2.096 2.245L23.75 24H4.25a2.25 2.25 0 0 1-2.245-2.096L2 21.75V6.25a2.25 2.25 0 0 1 2.096-2.245L4.25 4h5.957zm1.44 5.979a2.25 2.25 0 0 1-1.244.512l-.196.009l-6.707-.001V21.75c0 .38.282.694.648.743l.102.007h19.5a.75.75 0 0 0 .743-.648l.007-.102v-13a.75.75 0 0 0-.648-.743L23.75 8h-9.729l-2.374 1.979zM10.207 5.5H4.25a.75.75 0 0 0-.743.648L3.5 6.25v2.749L10.207 9a.75.75 0 0 0 .395-.113l.085-.06l1.891-1.578l-1.89-1.575a.75.75 0 0 0-.377-.167l-.104-.007z"
												fill="currentColor"
											></path></g
										></svg
									>
								</button>
							{:else}
								<span class="tooltip text-gray-300 dark:text-gray-600" data-tip="文件不存在">
									<svg
										class="h-6 w-6"
										xmlns="http://www.w3.org/2000/svg"
										xmlns:xlink="http://www.w3.org/1999/xlink"
										viewBox="0 0 28 28"
										><g fill="none"
											><path
												d="M10.207 4c.46 0 .908.141 1.284.402l.156.12L14.022 6.5h9.728a2.25 2.25 0 0 1 2.229 1.938l.016.158l.005.154v6.65a7.535 7.535 0 0 0-1.5-1.245V8.75a.75.75 0 0 0-.648-.743L23.75 8h-9.729l-2.374 1.979a2.25 2.25 0 0 1-1.244.512l-.196.009l-6.707-.001V21.75c0 .38.282.694.648.743l.102.007h9.02a7.45 7.45 0 0 0 .595 1.5H4.25a2.25 2.25 0 0 1-2.245-2.096L2 21.75V6.25a2.25 2.25 0 0 1 2.096-2.245L4.25 4h5.957zm0 1.5H4.25a.75.75 0 0 0-.743.648L3.5 6.25v2.749L10.207 9a.75.75 0 0 0 .395-.113l.085-.06l1.891-1.578l-1.89-1.575a.75.75 0 0 0-.377-.167l-.104-.007zM20.5 27a6.5 6.5 0 1 0 0-13a6.5 6.5 0 0 0 0 13zm0-1.5a4.978 4.978 0 0 1-2.965-.974l6.991-6.991A5 5 0 0 1 20.5 25.5zm2.965-9.026l-6.991 6.991a5 5 0 0 1 6.991-6.991z"
												fill="currentColor"
											></path></g
										></svg
									>
								</span>
							{/if}
						</td>
						<td class="min-w-16 text-sm font-medium">
							{#if row.status === RecordingStatus.Recording}
								<button
									class="tooltip"
									data-tip="停止录制"
									onclick={() => {
										openDialog(stopRecordDialogId);
										dialogUrl = row.url;
									}}
								>
									<svg
										class="h-6 w-6 hover:text-red-500"
										xmlns="http://www.w3.org/2000/svg"
										xmlns:xlink="http://www.w3.org/1999/xlink"
										viewBox="0 0 24 24"
										><g fill="none"
											><path
												d="M19.25 4.5a.25.25 0 0 1 .25.25v14.5a.25.25 0 0 1-.25.25H4.75a.25.25 0 0 1-.25-.25V4.75a.25.25 0 0 1 .25-.25h14.5zM4.75 3A1.75 1.75 0 0 0 3 4.75v14.5c0 .966.784 1.75 1.75 1.75h14.5A1.75 1.75 0 0 0 21 19.25V4.75A1.75 1.75 0 0 0 19.25 3H4.75z"
												fill="currentColor"
											></path></g
										></svg
									>
								</button>
							{:else}
								<button
									class="tooltip"
									data-tip="删除此记录"
									onclick={() => {
										deleteHistoryParams = {
											url: row.url,
											startTime: row.startTime,
											deleted: row.deleted
										};
										openDialog(deleteHistoryDialogId);
									}}
								>
									<svg
										class="h-6 w-6 hover:text-red-500"
										xmlns="http://www.w3.org/2000/svg"
										xmlns:xlink="http://www.w3.org/1999/xlink"
										viewBox="0 0 28 28"
										><g fill="none"
											><path
												d="M14 2.25a3.75 3.75 0 0 1 3.745 3.55l.005.2h5.5a.75.75 0 0 1 .102 1.493l-.102.007h-1.059l-1.22 15.053A3.75 3.75 0 0 1 17.233 26h-6.466a3.75 3.75 0 0 1-3.738-3.447L5.808 7.5H4.75a.75.75 0 0 1-.743-.648L4 6.75a.75.75 0 0 1 .648-.743L4.75 6h5.5A3.75 3.75 0 0 1 14 2.25zm6.687 5.25H7.313l1.211 14.932a2.25 2.25 0 0 0 2.243 2.068h6.466a2.25 2.25 0 0 0 2.243-2.068L20.686 7.5zm-8.937 3.75a.75.75 0 0 1 .743.648L12.5 12v8a.75.75 0 0 1-1.493.102L11 20v-8a.75.75 0 0 1 .75-.75zm4.5 0a.75.75 0 0 1 .743.648L17 12v8a.75.75 0 0 1-1.493.102L15.5 20v-8a.75.75 0 0 1 .75-.75zM14 3.75a2.25 2.25 0 0 0-2.245 2.096L11.75 6h4.5l-.005-.154A2.25 2.25 0 0 0 14 3.75z"
												fill="currentColor"
											></path></g
										></svg
									>
								</button>
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
