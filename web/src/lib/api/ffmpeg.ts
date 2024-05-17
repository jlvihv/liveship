import { API_URL } from '$lib';

// 检查 ffmpeg 是否可用
export async function checkFFmpeg(path: string) {
	// post /api/ffmpeg/check
	const response = await fetch(`${API_URL}/ffmpeg/check`, {
		method: 'POST',
		headers: {
			'Content-Type': 'application/json'
		},
		body: JSON.stringify({ path })
	});
	const data = await response.json();
	return data;
}
