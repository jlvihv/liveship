import { LiveStatus, PlatformKind, StreamingProtocol, type LiveInfo } from '@/model';
import { invoke } from '@tauri-apps/api/core';

export async function getLiveInfoForYoutube(url: string): Promise<LiveInfo> {
	let info: LiveInfo = {
		url,
		title: '',
		anchorName: '',
		anchorAvatar: '',
		viewerCount: '',
		roomCover: '',
		streams: [],
		platformKind: PlatformKind.Youtube,
		status: LiveStatus.NotLive
	};
	try {
		let youtubeInfo: any = await invoke('get_youtube_info', { url });
		info.status = youtubeInfo.videoDetails.isLiveContent ? LiveStatus.Live : LiveStatus.NotLive;
		info.viewerCount = youtubeInfo.videoDetails.viewCount;
		info.title = youtubeInfo.videoDetails.title;
		info.anchorName = youtubeInfo.videoDetails.author.name;
		info.anchorAvatar = youtubeInfo.videoDetails.author.thumbnails[0].url;
		info.roomCover = youtubeInfo.videoDetails.thumbnails[0].url || '';
		for (let format of youtubeInfo.formats) {
			info.streams.push({
				url: format.url,
				protocol: format.isHLS ? StreamingProtocol.Hls : StreamingProtocol.Flv,
				resolution: format.qualityLabel
			});
		}
	} catch (e) {
		console.error('get live info for youtube failed: ', e);
		// 抛出一个错误
		throw e;
	}
	return info;
}
