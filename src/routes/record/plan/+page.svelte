<script lang="ts">
	import { type RecordingPlan } from '$lib/model';
	import { invoke } from '@tauri-apps/api/core';
	import { onDestroy, onMount } from 'svelte';
	import { toast } from 'svelte-sonner';
	import Dialog from '$lib/components/dialog.svelte';
	import { closeDialog, getPlatformIcon, openDialog } from '@/utils';
	import { t } from '@/translations';
	import { scale } from 'svelte/transition';
	import { backOut } from 'svelte/easing';
	import Button from '@/components/button.svelte';

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
		try {
			list = await invoke('get_all_plans');
		} catch (e) {
			toast.error($t('getRecordPlanFailed'), {
				description: e as string
			});
		}
	}

	async function deletePlan(url: string) {
		closeDialog(deletePlanDialogId);
		dialogUrl = '';
		try {
			await invoke('delete_plan', { url });
			toast.success($t('deleteSuccess'));
			await getAllPlans();
		} catch (e) {
			toast.error($t('deleteFailed'), {
				description: e as string
			});
		}
	}

	async function updatePlanStatus(url: string, enabled: boolean) {
		try {
			await invoke('update_plan_status', { url, enabled });
			toast.success($t('updateSuccess'));
			await getAllPlans();
		} catch (e) {
			toast.error($t('updateFailed'), {
				description: e as string
			});
		}
	}

	// 获取最新一次轮询时间
	async function getLastPollingTime() {
		try {
			lastPollingTime = await invoke('get_last_polling_time');
		} catch (e) {
			toast.error($t('getLatestPollTimeFailed'), {
				description: e as string
			});
		}
	}
</script>

{#snippet icon(platformKind:string)}
	<div class="flex justify-center">
		<img class="h-6 w-6" src={getPlatformIcon(platformKind)} alt={platformKind} />
	</div>
{/snippet}

{#if list.length > 0}
	<div
		class="h-full w-full overflow-auto text-white2"
		transition:scale={{ duration: 300, easing: backOut, start: 0.9 }}
	>
		<table class="table">
			<thead>
				<tr>
					<th class="text-center">{$t('status')}</th>
					<th class="text-center">{$t('platform')}</th>
					<th class="min-w-20">{$t('anchor')}</th>
					<th>{$t('liveAddress')}</th>
					<th>{$t('type')}</th>
					<th>{$t('resolution')}</th>
					<th>{$t('action')}</th>
				</tr>
			</thead>
			<tbody>
				{#each list as row}
					<tr>
						<td>
							{#if row.enabled}
								<button
									class="tooltip w-full cursor-pointer text-green-500"
									data-tip={$t('clickToDisable')}
									onclick={() => updatePlanStatus(row.url, false)}
								>
									<span class="icon-[fluent--checkmark-circle-24-regular] h-6 w-6"></span>
								</button>
							{:else}
								<button
									class="tooltip w-full cursor-pointer"
									data-tip={$t('clickToEnable')}
									onclick={() => updatePlanStatus(row.url, true)}
								>
									<span class="icon-[fluent--dismiss-circle-28-regular] h-6 w-6 hover:text-white"
									></span>
								</button>
							{/if}
						</td>
						<td>
							{#if row.liveInfo?.platformKind}
								{@render icon(row.liveInfo?.platformKind)}
							{/if}
						</td>
						<td>{row.liveInfo?.anchorName}</td>
						<td>
							<a
								href={row.url}
								target="_blank"
								class="text-blue-600 transition duration-200 hover:text-blue-500"
							>
								<p class="w-32 truncate">
									{row.liveInfo?.title || row.url}
								</p>
							</a>
						</td>
						<td>{row.streamProtocol}</td>
						<td>{row.streamResolution !== '' ? $t(row.streamResolution) : $t('auto')}</td>
						<td
							><button
								class="tooltip"
								data-tip={$t('delete')}
								onclick={() => {
									openDialog(deletePlanDialogId);
									dialogUrl = row.url;
								}}
							>
								<span class="icon-[fluent--delete-24-regular] h-6 w-6 hover:text-red-500"></span>
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
	<Button
		white
		onClick={() => {
			closeDialog(deletePlanDialogId);
			dialogUrl = '';
		}}>{$t('cancel')}</Button
	>
	<Button red autoFocus onClick={() => deletePlan(dialogUrl)}>{$t('confirm')}</Button>
</Dialog>
