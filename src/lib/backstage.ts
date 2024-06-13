import { invoke } from '@tauri-apps/api/core';
import { LiveStatus, type RecordingPlan } from './model';
import { getLiveInfoForPlatform } from './utils';

export function checkPlanLoop() {
	// 进入时立刻检查一次计划
	checkPlans();
	// 然后每 60 秒检查一次
	setInterval(() => {
		checkPlans();
	}, 60000);
}

function checkPlans() {
	// 首先，获取有计划，但未在录制的任务
	invoke('get_plans_not_recording')
		.then((data) => {
			let plans: RecordingPlan[] = data as RecordingPlan[];
			if (plans.length == 0) {
				return;
			}
			// 遍历计划，检查主播是否已开播
			plans.forEach(async (plan) => {
				try {
					let liveInfo = await getLiveInfoForPlatform(plan.url);
					// 如果主播正在直播，则开始录制
					if (liveInfo.status === LiveStatus.Live) {
						if (liveInfo.streams.length == 0) {
							console.error('no stream found in live info: ', liveInfo);
							return;
						}
						// 根据 plan 里的分辨率和类型，选择一样的流，如果没有，选择一个最接近的
						let stream = liveInfo.streams[0];
						liveInfo.streams.forEach((s) => {
							if (s.resolution === plan.streamResolution && s.protocol === plan.streamProtocol) {
								stream = s;
							}
						});
						console.log('start record: ', liveInfo.anchorName, plan.url);
						// 开始录制
						invoke('start_record', {
							autoRecord: false,
							stream,
							liveInfo
						});
					}
				} catch (e) {
					console.error('check plan failed: ', e);
				}
			});
		})
		.catch((e) => {
			console.error('get plan failed: ', e);
		});
}
