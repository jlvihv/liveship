import { LiveStatus, PlatformKind, StreamingProtocol, type LiveInfo, type Stream } from '@/model';

export function getLiveInfoForKuaishou(url: string, html: string): LiveInfo {
	let info: LiveInfo = {
		url,
		title: '',
		anchorName: '',
		anchorAvatar: '',
		viewerCount: '',
		roomCover: '',
		streams: [],
		platformKind: PlatformKind.Bilibili,
		status: LiveStatus.NotLive
	};
	// match window.__NEPTUNE_IS_MY_WAIFU__=(.*?)window.__NEPTUNE_IS_MY_WAIFU__
	let jsonStr = html.match(/window.__NEPTUNE_IS_MY_WAIFU__=(.*?)window.__NEPTUNE_IS_MY_WAIFU__/);
	if (!jsonStr || jsonStr.length < 2) {
		return info;
	}
	let json = JSON.parse(jsonStr[1]);
	console.log('bilibili', json);

	return info;
}

export function getHeadersForKuaishou() {
	return {
		'User-Agent':
			'Mozilla/5.0 (Windows NT 10.0; Win64; x64; rv:109.0) Gecko/20100101 Firefox/115.0',
		Accept: 'text/html,application/xhtml+xml,application/xml;q=0.9,image/avif,image/webp,*/*;q=0.8',
		Referer: 'https://live.bilibili.com/?spm_id_from=333.1296.0.0'
	};
}
