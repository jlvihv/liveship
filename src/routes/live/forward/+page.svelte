<script lang="ts">
	import { toast } from 'svelte-sonner';
	import { debounce, getLiveInfoForPlatform } from '$lib/utils';
	import { onMount } from 'svelte';
	import { invoke } from '@tauri-apps/api/core';
	import { LiveStatus, type LiveInfo } from '$lib/model';
	import { t } from '@/translations';
	import Button from '$lib/components/button.svelte';
	import { buildFFmpegForwardCommand } from '@/ffmpeg';
	import { scale } from 'svelte/transition';
	import { backOut } from 'svelte/easing';

	// 关键信息变量

	let url = $state('');
	let id = $state(0);
	const xiaohongshuPushServer = 'rtmp://live-push.xhscdn.com/live';
	let liveInfo: LiveInfo | undefined = $state();
	let errorMessage = $state('');
	let streamUrl = $state('');
	let key = $state('');
	let outputUrl = $derived(`${xiaohongshuPushServer}/${key}`);
	let forwarding = $state(false);

	// ui 状态变量

	// 用一个变量来表示是否正在请求
	let requesting = $state(false);
	let visible = $state(false);

	onMount(() => {
		visible = true;
	});

	// 防抖调用 api, 500ms 内只调用一次
	const handleinput = debounce(async () => {
		getLiveInfo(url);
	}, 500);

	async function getLiveInfo(url: string) {
		if (!url) {
			return;
		}
		// 先清空之前的数据
		liveInfo = undefined;
		errorMessage = '';

		requesting = true;

		try {
			liveInfo = await getLiveInfoForPlatform(url);
			if (liveInfo.streams.length > 0) {
				streamUrl = liveInfo.streams[0].url;
			}
		} catch (err) {
			requesting = false;
			errorMessage = err as string;
		}

		requesting = false;
	}

	async function forwardStream() {
		if (!streamUrl || !outputUrl) {
			toast.error($t('streamOrOutputUrlEmpty'));
			return;
		}
		let ffmpegCommand = buildFFmpegForwardCommand(streamUrl, outputUrl);
		try {
			id = await invoke('execute_ffmpeg_command', { ffmpegCommand });
			// 只要不异常，就认为成功了
			forwarding = true;
			toast.success('已经开始转发啦');
		} catch (e) {
			toast.error(e as string);
			forwarding = false;
		}
	}

	async function stopForward() {
		try {
			await invoke('kill_child', { id });
			forwarding = false;
			toast.success('已经停止转发啦');
		} catch (e) {
			toast.error(e as string);
		}
	}
</script>

{#if visible}
	<div
		class="flex h-screen flex-col items-center justify-start"
		transition:scale={{ duration: 300, easing: backOut, start: 0.9 }}
	>
		<div
			class="w-1/2 min-w-[600px] transform-gpu transition-all duration-500 ease-out {liveInfo
				? 'mt-16'
				: 'mt-32'}"
		>
			<label class="flex h-14 rounded-full bg-gray1 px-2 text-gray2 forced-color-adjust-none">
				<input
					oninput={handleinput}
					bind:value={url}
					class="m-0 grow resize-none appearance-none overflow-hidden border-none bg-transparent px-0 py-4 pl-4 placeholder-gray2 outline-none focus:text-white1"
					placeholder={$t('inputPlaceholder')}
				/>
				{#if url}
					<button
						class="tooltip mr-2 flex items-center"
						data-tip={$t('clear')}
						onclick={() => {
							url = '';
							liveInfo = undefined;
							errorMessage = '';
						}}
					>
						<span class="icon-[fluent--dismiss-circle-28-regular] h-6 w-6 text-gray-500"></span>
					</button>
				{/if}
			</label>
		</div>
		<div class="w-1/2 min-w-[600px]">
			{#if errorMessage || liveInfo || requesting}
				<div class="flex items-center justify-between pt-8">
					{#if liveInfo?.status === LiveStatus.Live}
						<h1 class="text-2xl font-bold text-white1">
							{liveInfo?.title}
						</h1>
					{:else if liveInfo?.status === LiveStatus.NotLive}
						<div>
							<h1 class="text-2xl font-bold text-white1">
								{$t('anchor')}
								{liveInfo.anchorName}
								{$t('notInLive')}
							</h1>
						</div>
					{:else if errorMessage}
						<div class="text-red-500">{errorMessage}</div>
					{:else}
						<!-- 空标签占位 -->
						<div></div>
					{/if}

					<button class="px-3" onclick={() => getLiveInfo(url)}>
						<span
							class="icon-[fluent--arrow-clockwise-28-regular] h-8 w-8 {requesting ? 'rotate' : ''}"
						></span>
					</button>
				</div>

				{#if requesting}
					<div class="flex h-full w-full items-center justify-center">
						<span class="loading loading-dots loading-md"></span>
					</div>
				{/if}
				{#if liveInfo}
					<div>
						<div class="w-full text-white1">
							{#if liveInfo.status === LiveStatus.Live}
								<div class="flex justify-between">
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
													<p class="text-green-500">{$t('living')}</p>
												{:else}
													<p class="text-gray-500">{$t('notLiving')}</p>
												{/if}
												<p>
													{liveInfo.viewerCount ? liveInfo.viewerCount + $t('watching') : ''}
												</p>
											</div>
										</div>
									</div>
									<div class="flex items-center px-4">
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
								</div>

								<div class="pt-8">
									<label
										class="flex h-14 rounded-full bg-gray1 px-2 text-gray2 forced-color-adjust-none"
									>
										<span class="flex h-full items-center border-r-2 border-dark px-4">
											{$t('xiaohongshu')}
										</span>
										<input
											bind:value={key}
											class="m-0 grow resize-none appearance-none overflow-hidden border-none bg-transparent px-0 py-4 pl-4 placeholder-gray2 outline-none focus:text-white1"
											placeholder="在这里输入小红书直播串流密钥"
										/>
										{#if outputUrl}
											<button
												class="tooltip mr-2 flex items-center"
												data-tip={$t('clear')}
												onclick={() => {
													key = '';
												}}
											>
												<span class="icon-[fluent--dismiss-circle-28-regular] h-6 w-6 text-gray-500"
												></span>
											</button>
										{/if}
									</label>
									{#if key}
										<div class="flex justify-center pt-8">
											{#if !forwarding}
												<Button white className="w-2/3" onClick={() => forwardStream()}
													>{$t('startForward')}</Button
												>
											{:else}
												<Button red className="w-2/3" onClick={() => stopForward()}
													>{$t('stopForward')}</Button
												>
											{/if}
										</div>
									{/if}
								</div>
							{:else}
								<div class="pt-8">
									<p>需主播开播后，才可启动直播转发</p>
								</div>
							{/if}
						</div>
					</div>
				{/if}
			{/if}
			<p class="pt-8 text-sm">{$t('streamForwardWarning')}</p>
		</div>
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
