import { PlatformKind, type LiveInfo } from './model';
import { getLiveInfoForDouyin } from './platform/douyin';
import { getLiveInfoForHuya } from './platform/huya';
import { getLiveInfoForKuaishou } from './platform/kuaishou';
import { getLiveInfoForTiktok } from './platform/tiktok';
import { getLiveInfoForTwitch } from './platform/twitch';
import { getLiveInfoForXiaohongshu } from './platform/xiaohongshu';
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
			return '/douyin.ico';
		case 'huya':
			return '/huya.ico';
		case 'bilibili':
			return '/bilibili.ico';
		case 'douyu':
			return '/douyu.ico';
		case 'kuaishou':
			return '/kuaishou.ico';
		case 'twitch':
			return '/twitch.ico';
		case 'youtube':
			return '/youtube.png';
		case 'xiaohongshu':
			return '/xiaohongshu.ico';
		case 'tiktok':
			return '/tiktok.ico';
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
		case PlatformKind.Xiaohongshu:
			return getLiveInfoForXiaohongshu(url);
		case PlatformKind.Kuaishou:
			return getLiveInfoForKuaishou(url);
		default:
			throw new Error('unknown platform');
	}
}
