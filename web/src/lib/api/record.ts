import { API_URL } from '$lib';
import type { Stream } from '$lib/model';

// 开始录制
export async function startRecord(
	url: string,
	autoRecord: boolean,
	stream: Stream,
	platformKind: string,
	anchorName: string
) {
	// post /api/record/start
	const response = await fetch(`${API_URL}/record/start`, {
		method: 'POST',
		headers: {
			'Content-Type': 'application/json'
		},
		body: JSON.stringify({ url, autoRecord, stream, platformKind, anchorName })
	});
	const data = await response.json();
	return data;
}

// 停止录制
export async function stopRecord(url: string) {
	// post /api/record/stop
	const response = await fetch(`${API_URL}/record/stop`, {
		method: 'POST',
		headers: {
			'Content-Type': 'application/json'
		},
		body: JSON.stringify({ url })
	});
	const data = await response.json();
	return data;
}

// 获取录制状态
export async function getRecordStatus(url: string) {
	// post /api/record/status
	const response = await fetch(`${API_URL}/record/status`, {
		method: 'POST',
		headers: {
			'Content-Type': 'application/json'
		},
		body: JSON.stringify({ url })
	});
	const data = await response.json();
	return data;
}
