<script lang="ts">
	import { deletePlan, getAllPlan, getLastPollingTime, updatePlanStatus } from '$lib/api/plan';
	import { type ApiResponse, type RecordingPlan } from '$lib/model';
	import { onDestroy, onMount } from 'svelte';
	import { toast } from 'svelte-sonner';

	let list: RecordingPlan[] = $state([]);
	let lastPollingTime = $state(0); // 时间戳毫秒
	let intervalId: number | undefined = $state();

	async function getPlan() {
		const resp: ApiResponse = await getAllPlan();
		if (resp.code == 0) {
			list = resp.data as RecordingPlan[];
		} else {
			toast.error('获取录制计划失败', {
				description: resp.message
			});
		}
	}

	onMount(async () => {
		await getPlan();
		// 获取最新一次轮询时间，每隔 1 分钟再获取一次
		await fetchGetLastPollingTime();
		intervalId = setInterval(fetchGetLastPollingTime, 60000);
	});

	// 组件销毁时，清除定时器
	onDestroy(() => {
		if (intervalId) {
			clearInterval(intervalId);
		}
	});

	async function fetchDeletePlan(url: string) {
		const resp: ApiResponse = await deletePlan(url);
		if (resp.code == 0) {
			toast.success('删除成功');
			await getPlan();
		} else {
			toast.error('删除失败', {
				description: resp.message
			});
		}
	}

	async function updateRecordingPlanStatus(url: string, enabled: boolean) {
		const resp: ApiResponse = await updatePlanStatus(url, enabled);
		if (resp.code == 0) {
			toast.success('更新成功');
			await getPlan();
		} else {
			toast.error('更新失败', {
				description: resp.message
			});
		}
	}

	// 获取最新一次轮询时间
	async function fetchGetLastPollingTime() {
		const resp: ApiResponse = await getLastPollingTime();
		if (resp.code == 0) {
			lastPollingTime = resp.data as number;
		} else {
			toast.error('获取最新轮询时间失败', {
				description: resp.message
			});
		}
	}
</script>

{#if list.length > 0}
	<div class="overflow-x-auto">
		<table class="table table-zebra">
			<thead>
				<tr>
					<th>主播</th>
					<th>直播间</th>
					<th>是否启用</th>
					<th>直播流类型</th>
					<th>录制分辨率</th>
					<th>操作</th>
				</tr>
			</thead>
			<tbody>
				{#each list as row}
					<tr>
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
						<td>
							{#if row.enabled}
								<button
									class="tooltip cursor-pointer text-green-500"
									data-tip="点击以禁用"
									onclick={() => updateRecordingPlanStatus(row.url, false)}>已启用</button
								>
							{:else}
								<button
									class="tooltip cursor-pointer text-gray-400 dark:text-gray-500"
									data-tip="点击以启用"
									onclick={() => updateRecordingPlanStatus(row.url, true)}>已禁用</button
								>
							{/if}
						</td>
						<td>{row.streamKind}</td>
						<td>{row.streamResolution}</td>
						<td class="text-sm font-medium"
							><button
								class="text-indigo-600 hover:text-indigo-900"
								onclick={() => fetchDeletePlan(row.url)}>删除</button
							></td
						>
					</tr>
				{/each}
			</tbody>
		</table>
	</div>
{:else}
	<div class="flex h-full w-full items-center justify-center">
		<p>录制计划为空</p>
	</div>
{/if}
