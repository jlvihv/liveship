import { API_URL } from '$lib';

// 获取所有录制历史
export async function getAllRecordingHistory() {
	// get /api/history
	const response = await fetch(`${API_URL}/history`);
	const data = await response.json();
	return data;
}

// 删除一条历史记录，需要提供 url 和 startTime
export async function deleteRecordingHistory(url: string, startTime: number) {
	// delete /api/history
	const response = await fetch(`${API_URL}/history`, {
		method: 'DELETE',
		headers: {
			'Content-Type': 'application/json'
		},
		body: JSON.stringify({ url, startTime })
	});
	const data = await response.json();
	return data;
}
