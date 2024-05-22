import { API_URL } from '$lib';

// 新建录制计划
export async function addPlan(url: string) {
	// post /api/plan
	const response = await fetch(`${API_URL}/plan`, {
		method: 'POST',
		headers: {
			'Content-Type': 'application/json'
		},
		body: JSON.stringify({ url })
	});
	const data = await response.json();
	return data;
}

// 删除录制计划
export async function deletePlan(url: string) {
	// delete /api/plan
	const response = await fetch(`${API_URL}/plan`, {
		method: 'DELETE',
		headers: {
			'Content-Type': 'application/json'
		},
		body: JSON.stringify({ url })
	});
	const data = await response.json();
	return data;
}

// 获取所有录制计划
export async function getAllPlan() {
	// get /api/plan
	const response = await fetch(`${API_URL}/plan`);
	const data = await response.json();
	return data;
}

// 更新计划状态
export async function updatePlanStatus(url: string, enabled: boolean) {
	// put /api/plan/enable or /api/plan/disable
	const response = await fetch(`${API_URL}/plan/${enabled ? 'enable' : 'disable'}`, {
		method: 'PUT',
		headers: {
			'Content-Type': 'application/json'
		},
		body: JSON.stringify({ url })
	});
	const data = await response.json();
	return data;
}

// 获取最新一次的轮询时间
export async function getLastPollingTime() {
	// get /api/plan/lasttime
	const response = await fetch(`${API_URL}/plan/lasttime`);
	const data = await response.json();
	return data;
}
