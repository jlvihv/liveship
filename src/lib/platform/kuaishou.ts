import { LiveStatus, PlatformKind, StreamingProtocol, type LiveInfo, type Stream } from '@/model';

export function getLiveInfoForKuaishou(url: string): LiveInfo {
	let info: LiveInfo = {
		url,
		title: '',
		anchorName: '',
		anchorAvatar: '',
		viewerCount: '',
		roomCover: '',
		streams: [],
		platformKind: PlatformKind.Kuaishou,
		status: LiveStatus.NotLive
	};
	return info;
}

export function getHeadersForKuaishou() {
	return {
		'User-Agent':
			'Mozilla/5.0 (Windows NT 10.0; Win64; x64; rv:109.0) Gecko/20100101 Firefox/115.0',
		Accept: 'text/html,application/xhtml+xml,application/xml;q=0.9,image/avif,image/webp,*/*;q=0.8'
	};
}
