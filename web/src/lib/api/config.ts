import { API_URL } from '$lib';
import type { AppConfig } from '$lib/model';

// 获取配置
export async function getConfig() {
	// get /api/config
	const response = await fetch(`${API_URL}/config`);
	const data = await response.json();
	return data;
}

// 设置配置
export async function setConfig(config: AppConfig) {
	// post /api/config
	const response = await fetch(`${API_URL}/config`, {
		method: 'POST',
		headers: {
			'Content-Type': 'application/json'
		},
		body: JSON.stringify(config)
	});
	const data = await response.json();
	return data;
}
