<script lang="ts">
	import { RecordingStatus, type RecordingHistory, type RecordingPlan } from '$lib/model';
	import Button from '$lib/components/button.svelte';
	import { scale } from 'svelte/transition';
	import { onDestroy, onMount } from 'svelte';
	import { toast } from 'svelte-sonner';
	import dayjs from 'dayjs';
	import relativeTime from 'dayjs/plugin/relativeTime';
	import 'dayjs/locale/zh-cn';
	import Dialog from '$lib/components/dialog.svelte';
	import { closeDialog, openDialog, formatFileSize, getPlatformIcon } from '$lib/utils';
	import { invoke } from '@tauri-apps/api/core';
	import { t } from '@/translations';
	import { backOut } from 'svelte/easing';
	dayjs.extend(relativeTime);

	// icon fluent 28

	let stopRecordDialogId = 'stopRecord';
	let deleteHistoryDialogId = 'deleteHistory';
	let list: RecordingHistory[] = $state([]);
	let intervalId: number | undefined = $state();
	let deleteHistoryParams: { url: string; startTime: number; deleted: boolean } | undefined =
		$state();
	let stopRecordParams: { url: string; inPlan: boolean } | undefined = $state();

	// 组件挂载时，获取录制历史，并每隔两秒获取一次
	onMount(async () => {
		// 先获取一次，每隔两秒，再获取一次录制历史
		await getAllHistory();
		intervalId = setInterval(getAllHistory, 2000);

		// 从 localStorage 中获取 lang，设置 dayjs 的语言
		const lang = localStorage.getItem('lang');
		if (lang) {
			if (lang === 'cn') {
				dayjs.locale('zh-cn');
			}
		}
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
				toast.error($t('getRecordHistoryFailed'), {
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
				toast.success($t('deleteSuccess'));
				getAllHistory();
			})
			.catch((e) => {
				toast.error($t('deleteSuccess'), {
					description: e
				});
			});
	}

	async function stopRecord(url: string, disablePlan: boolean) {
		stopRecordParams = undefined;
		closeDialog(stopRecordDialogId);
		if (url === '') {
			toast.error($t('urlCannotBeEmpty'));
			return;
		}
		if (disablePlan) {
			// 禁用此计划
			invoke('update_plan_status', { url, enabled: false })
				.then(() => {})
				.catch((e) => {
					toast.error($t('disablePlanFailed'), {
						description: e
					});
				});
		}
		invoke('stop_record', { url })
			.then(() => {
				toast.success($t('recordAlreadyStopped'));
				getAllHistory();
			})
			.catch((e) => {
				toast.error($t('recordStopFailed'), {
					description: e
				});
			});
	}

	// 在文件管理器中打开文件夹
	async function openInFolder(path: string) {
		invoke('open_in_folder', { path })
			.then(() => {
				toast.success($t('openedInFileManager'));
			})
			.catch((e) => {
				toast.error($t('openInFileManagerFailed'), {
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
</script>

<!-- 原生对话框 -->
<Dialog
	id={stopRecordDialogId}
	text={stopRecordParams?.inPlan ? $t('confirmStopRecordAndDisablePlan') : $t('confirmStopRecord')}
>
	<Button
		white
		onClick={() => {
			closeDialog(stopRecordDialogId);
			stopRecordParams = undefined;
		}}>{$t('cancel')}</Button
	>
	{#if stopRecordParams?.inPlan}
		<Button red onClick={() => stopRecord(stopRecordParams?.url || '', true)}
			>{$t('stopRecordAndDisablePlan')}</Button
		>
	{/if}
	<Button autoFocus red onClick={() => stopRecord(stopRecordParams?.url || '', false)}
		>{stopRecordParams?.inPlan ? $t('onlyStopRecord') : $t('confirm')}</Button
	>
</Dialog>

<Dialog
	id={deleteHistoryDialogId}
	text={deleteHistoryParams?.deleted ? $t('confirmDeleteRecord') : $t('deleteRecordOnlyOrFile')}
>
	<Button
		white
		className=""
		onClick={() => {
			closeDialog(deleteHistoryDialogId);
			deleteHistoryParams = undefined;
		}}>{$t('cancel')}</Button
	>
	{#if !deleteHistoryParams?.deleted}
		<Button
			red
			onClick={() => deleteHistory(
			  deleteHistoryParams!.url,
    deleteHistoryParams!.startTime,
    true
			)}
			>{$t('deleteRecordAndFile')}</Button
		>
	{/if}
	<!-- svelte-ignore a11y_autofocus -->
	<Button
		autoFocus
		red
		onClick={() =>
			deleteHistory(
				deleteHistoryParams!.url,
				deleteHistoryParams!.startTime,
				false
			)}
		>{deleteHistoryParams?.deleted ? $t('confirm') : $t('deleteRecordOnly')}</Button
	>
</Dialog>

{#snippet icon(platformKind:string)}
	<div class="flex justify-center">
		<img class="h-6 w-6" src={getPlatformIcon(platformKind)} alt={platformKind} />
	</div>
{/snippet}

{#if list.length > 0}
	<div
		class="overflow-auto text-white2"
		transition:scale={{ duration: 300, easing: backOut, start: 0.9 }}
	>
		<table class="table">
			<thead>
				<tr>
					<th class="text-center">{$t('status')}</th>
					<th class="text-center">{$t('platform')}</th>
					<th>{$t('anchor')}</th>
					<th>{$t('liveAddress')}</th>
					<th>{$t('duration')}</th>
					<th class="min-w-24">{$t('size')}</th>
					<th>{$t('file')}</th>
					<th>{$t('action')}</th>
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
										: 'bg-gray-600'}"
									data-tip={row.status == RecordingStatus.Recording
										? $t('recording')
										: $t('recorded')}
								>
								</span>
							</p>
						</td>
						<td>
							{@render icon(row.liveInfo?.platformKind || '')}
						</td>
						<td>
							<p class="w-32 truncate">
								{row.liveInfo?.anchorName}
							</p>
						</td>
						<td
							><a
								href={row.url}
								target="_blank"
								class="text-blue-600 transition duration-200 hover:text-blue-500"
							>
								<p class="w-32 truncate">
									{row.liveInfo?.title || row.url}
								</p>
							</a></td
						>
						<td>
							<p class="w-16 truncate">
								{calcDuration(row.startTime, row.endTime)}
							</p>
						</td>
						<td>{formatFileSize(row.fileSize)}</td>
						<td>
							{#if !row.deleted}
								<button
									onclick={() => openInFolder(row.path)}
									class="tooltip text-left hover:text-white"
									data-tip={$t('openInFileManager')}
								>
									<span class="icon-[fluent--folder-32-regular] h-6 w-6"></span>
								</button>
							{:else}
								<span class="tooltip text-gray-600" data-tip={$t('fileNotExist')}>
									<span class="icon-[fluent--dismiss-circle-28-regular] h-6 w-6 text-gray-500"
									></span>
								</span>
							{/if}
						</td>
						<td>
							{#if row.status === RecordingStatus.Recording}
								<button
									class="tooltip"
									data-tip={$t('stop')}
									onclick={async () => {
										stopRecordParams = {
											url: row.url,
											inPlan: await isInPlan(row.url)
										};
										openDialog(stopRecordDialogId);
									}}
								>
									<span class="icon-[fluent--record-stop-28-regular] h-6 w-6 hover:text-red-500"
									></span>
								</button>
							{:else}
								<button
									class="tooltip"
									data-tip={$t('delete')}
									onclick={() => {
										deleteHistoryParams = {
											url: row.url,
											startTime: row.startTime,
											deleted: row.deleted
										};
										openDialog(deleteHistoryDialogId);
									}}
								>
									<span class="icon-[fluent--delete-24-regular] h-6 w-6 hover:text-red-500"></span>
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
		<p>{$t('recordHistoryEmpty')}</p>
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
