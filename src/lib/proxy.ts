import { invoke } from '@tauri-apps/api/core';
import type { ProxyConfig } from './model';

let proxyConfig: ProxyConfig | null = null;

export async function getProxyConfig(): Promise<ProxyConfig | null> {
	if (proxyConfig === null) {
		try {
			proxyConfig = await invoke('get_system_proxy_config');
			return proxyConfig;
		} catch (e) {
			console.error('get proxy info failed: ', e);
			return null;
		}
	} else {
		return proxyConfig;
	}
}

export async function getProxyForFetch(): Promise<string | undefined> {
	let config = await getProxyConfig();
	if (config === null) {
		return undefined;
	}
	if (config.enabled) {
		return config.address;
	} else {
		return undefined;
	}
}
