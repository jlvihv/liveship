import { API_URL } from '$lib';

// 获取直播信息
export async function getLiveInfo(url: string) {
	// post /api/stream-info
	const response = await fetch(`${API_URL}/live/info`, {
		method: 'POST',
		headers: {
			'Content-Type': 'application/json'
		},
		body: JSON.stringify({ url })
	});
	const data = await response.json();
	return data;
}
