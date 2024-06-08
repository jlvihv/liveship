<script lang="ts">
	import { type RecordingPlan } from '$lib/model';
	import { invoke } from '@tauri-apps/api/core';
	import { onDestroy, onMount } from 'svelte';
	import { toast } from 'svelte-sonner';
	import Dialog from './dialog.svelte';
	import { closeDialog, getPlatformIcon, openDialog } from '@/utils';
	import { t } from '@/translations';

	const deletePlanDialogId = 'deletePlan';
	let list: RecordingPlan[] = $state([]);
	let lastPollingTime = $state(0); // 时间戳毫秒
	let intervalId: number | undefined = $state();
	let dialogUrl = $state('');

	onMount(async () => {
		await getAllPlans();
		// 获取最新一次轮询时间，每隔 1 分钟再获取一次
		await getLastPollingTime();
		intervalId = setInterval(getLastPollingTime, 60000);
	});

	// 组件销毁时，清除定时器
	onDestroy(() => {
		if (intervalId) {
			clearInterval(intervalId);
		}
	});

	async function getAllPlans() {
		invoke('get_all_plans')
			.then((data) => {
				list = data as RecordingPlan[];
			})
			.catch((e) => {
				toast.error($t('getRecordPlanFailed'), {
					description: e
				});
			});
	}

	async function deletePlan(url: string) {
		closeDialog(deletePlanDialogId);
		dialogUrl = '';
		invoke('delete_plan', { url })
			.then(() => {
				toast.success($t('deleteSuccess'));
				getAllPlans();
			})
			.catch((e) => {
				toast.error($t('deleteFailed'), {
					description: e
				});
			});
	}

	async function updatePlanStatus(url: string, enabled: boolean) {
		invoke('update_plan_status', { url, enabled })
			.then(() => {
				toast.success($t('updateSuccess'));
				getAllPlans();
			})
			.catch((e) => {
				toast.error($t('updateFailed'), {
					description: e
				});
			});
	}

	// 获取最新一次轮询时间
	async function getLastPollingTime() {
		invoke('get_last_polling_time')
			.then((data) => {
				lastPollingTime = data as number;
			})
			.catch((e) => {
				toast.error($t('getLatestPollTimeFailed'), {
					description: e
				});
			});
	}
</script>

{#snippet icon(platformKind:string)}
	<img class="h-6 w-6" src={getPlatformIcon(platformKind)} alt={platformKind} />
{/snippet}

{#if list.length > 0}
	<div class="h-full w-full overflow-auto">
		<table class="table table-zebra">
			<thead>
				<tr>
					<th>{$t('platform')}</th>
					<th class="min-w-20">{$t('anchor')}</th>
					<th>{$t('liveAddress')}</th>
					<th>{$t('type')}</th>
					<th>{$t('resolution')}</th>
					<th>{$t('status')}</th>
					<th class="min-w-24">{$t('action')}</th>
				</tr>
			</thead>
			<tbody>
				{#each list as row}
					<tr>
						<td>
							{#if row.liveInfo?.platformKind}
								{@render icon(row.liveInfo?.platformKind)}
							{/if}
						</td>
						<td>{row.liveInfo?.anchorName}</td>
						<td>
							{#if row.liveInfo && row.liveInfo!.title}
								<a
									href={row.url}
									target="_blank"
									class="text-blue-500 transition duration-200 hover:text-blue-700"
									>{row.liveInfo?.title}</a
								>
							{:else}
								<a
									href={row.url}
									target="_blank"
									class="text-blue-500 transition duration-200 hover:text-blue-700">{row.url}</a
								>
							{/if}
						</td>
						<td>{row.streamProtocol}</td>
						<td>{row.streamResolution !== '' ? row.streamResolution : $t('auto')}</td>
						<td>
							{#if row.enabled}
								<button
									class="tooltip cursor-pointer text-green-500"
									data-tip={$t('clickToDisable')}
									onclick={() => updatePlanStatus(row.url, false)}
								>
									<svg
										class="h-6 w-6"
										xmlns="http://www.w3.org/2000/svg"
										xmlns:xlink="http://www.w3.org/1999/xlink"
										viewBox="0 0 24 24"
										><g fill="none"
											><path
												d="M12 2c5.523 0 10 4.477 10 10s-4.477 10-10 10S2 17.523 2 12S6.477 2 12 2zm0 1.5a8.5 8.5 0 1 0 0 17a8.5 8.5 0 0 0 0-17zm-1.25 9.94l4.47-4.47a.75.75 0 0 1 1.133.976l-.073.084l-5 5a.75.75 0 0 1-.976.073l-.084-.073l-2.5-2.5a.75.75 0 0 1 .976-1.133l.084.073l1.97 1.97l4.47-4.47l-4.47 4.47z"
												fill="currentColor"
											></path></g
										></svg
									>
								</button>
							{:else}
								<button
									class="tooltip cursor-pointer text-gray-400 dark:text-gray-500"
									data-tip={$t('clickToEnable')}
									onclick={() => updatePlanStatus(row.url, true)}
								>
									<svg
										class="h-6 w-6"
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
						</td>
						<td
							><button
								class="tooltip"
								data-tip={$t('deleteThisPlan')}
								onclick={() => {
									openDialog(deletePlanDialogId);
									dialogUrl = row.url;
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
							</button></td
						>
					</tr>
				{/each}
			</tbody>
		</table>
	</div>
{:else}
	<div class="flex h-full w-full items-center justify-center">
		<p>{$t('recordPlanEmpty')}</p>
	</div>
{/if}

<!-- 原生对话框 -->
<Dialog text={$t('confirmDeletePlan')} id={deletePlanDialogId}>
	<button
		class="btn w-24"
		onclick={() => {
			closeDialog(deletePlanDialogId);
			dialogUrl = '';
		}}>{$t('cancel')}</button
	>
	<!-- svelte-ignore a11y_autofocus -->
	<button autofocus class="btn btn-primary w-24" onclick={() => deletePlan(dialogUrl)}
		>{$t('confirm')}</button
	>
</Dialog>
