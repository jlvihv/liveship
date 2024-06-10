import { PlatformKind, type LiveInfo } from './model';
import { getLiveInfoForDouyin } from './platform/douyin';
import { getLiveInfoForHuya } from './platform/huya';
import { getLiveInfoForTiktok } from './platform/tiktok';
import { getLiveInfoForTwitch } from './platform/twitch';
import { getLiveInfoForYoutube } from './platform/youtube';

// eslint-disable-next-line @typescript-eslint/no-explicit-any
export function debounce<F extends (...args: any[]) => any>(
	fn: F,
	delay: number
): (...args: Parameters<F>) => void {
	let timeoutId: ReturnType<typeof setTimeout> | null = null;

	return (...args: Parameters<F>): void => {
		if (timeoutId) {
			clearTimeout(timeoutId);
		}

		timeoutId = setTimeout(() => {
			fn(...args);
		}, delay);
	};
}

// eslint-disable-next-line @typescript-eslint/no-explicit-any
export function throttle<F extends (...args: any[]) => any>(
	fn: F,
	delay: number
): (...args: Parameters<F>) => void {
	let lastCall = 0;

	return (...args: Parameters<F>): void => {
		const now = new Date().getTime();
		if (now - lastCall < delay) {
			return;
		}
		lastCall = now;
		fn(...args);
	};
}

// 获取分辨率的中文表示
export function getResolutionName(resolution: string) {
	switch (resolution.toLowerCase()) {
		case 'full_hd1':
		case 'origin':
			return '原画';
		case 'hd1':
		case 'uhd':
		case 'hd':
			return '超清';
		case 'hd-60':
			return '超清 60 帧';
		case 'sd1':
			return '高清';
		case 'sd2':
		case 'sd':
			return '标清';
		case 'ld':
			return '流畅';
		case 'default':
			return '默认';
		case 'ao':
			return '仅音频';
		default:
			return resolution;
	}
}

// 格式化显示文件尺寸
export function formatFileSize(bytes: number) {
	if (bytes === 0) return '0 B';

	const k = 1024;
	const sizes = ['B', 'KB', 'MB', 'GB', 'TB', 'PB', 'EB', 'ZB', 'YB'];
	const i = Math.floor(Math.log(bytes) / Math.log(k));

	return `${(bytes / Math.pow(k, i)).toFixed(1)} ${sizes[i]}`;
}

export function closeDialog(id: string) {
	let dialog = document.getElementById(id) as HTMLDialogElement;
	dialog?.close();
}

export function openDialog(id: string) {
	let dialog = document.getElementById(id) as HTMLDialogElement;
	dialog?.showModal();
}

export function getPlatformIcon(platformKind: string): string {
	switch (platformKind.toLowerCase()) {
		case 'douyin':
			return 'https://www.douyin.com/favicon.ico';
		case 'huya':
			return 'https://www.huya.com/favicon.ico';
		case 'bilibili':
			return 'https://www.bilibili.com/favicon.ico';
		case 'douyu':
			return 'https://www.douyu.com/favicon.ico';
		case 'kuaishou':
			return 'https://m.kuaishou.com/favicon.ico';
		case 'twitch':
			return 'https://m.twitch.tv/favicon.ico?desktop-redirect=true';
		case 'youtube':
			return 'https://m.youtube.com/static/apple-touch-icon-72x72-precomposed.png';
		case 'tiktok':
			return 'https://www.tiktok.com/favicon.ico';
		default:
			return 'https://www.google.com/favicon.ico';
	}
}

// 获取对应平台的 PlatformKind
export function getPlatformKind(url: string): PlatformKind {
	url = url.toLowerCase();
	switch (true) {
		case url.startsWith('https://live.douyin.com/') || url.startsWith('https://v.douyin.com/'):
			return PlatformKind.Douyin;
		case url.startsWith('https://www.tiktok.com/'):
			return PlatformKind.Tiktok;
		case url.startsWith('https://www.xiaohongshu.com/'):
			return PlatformKind.Xiaohongshu;
		case url.startsWith('https://www.huya.com/'):
			return PlatformKind.Huya;
		case url.startsWith('https://www.twitch.tv/'):
			return PlatformKind.Twitch;
		case url.startsWith('https://www.youtube.com/watch?v='):
			return PlatformKind.Youtube;
		default:
			return PlatformKind.Unknown;
	}
}

// 根据 url 获取对应平台的直播信息，先获取 PlatformKind，再获取对应的直播信息
export async function getLiveInfoForPlatform(url: string): Promise<LiveInfo> {
	let platformKind = getPlatformKind(url);
	switch (platformKind) {
		case PlatformKind.Douyin:
			return getLiveInfoForDouyin(url);
		case PlatformKind.Tiktok:
			return getLiveInfoForTiktok(url);
		case PlatformKind.Huya:
			return getLiveInfoForHuya(url);
		case PlatformKind.Twitch:
			return getLiveInfoForTwitch(url);
		case PlatformKind.Youtube:
			return getLiveInfoForYoutube(url);
		default:
			throw new Error('unknown platform');
	}
}
