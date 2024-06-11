import { LiveStatus, PlatformKind, StreamingProtocol, type LiveInfo } from '@/model';
import { invoke } from '@tauri-apps/api/core';

export async function getLiveInfoForXiaohongshu(url: string): Promise<LiveInfo> {
	let info: LiveInfo = {
		url,
		title: '',
		anchorName: '',
		anchorAvatar: '',
		viewerCount: '',
		roomCover: '',
		streams: [],
		platformKind: PlatformKind.Xiaohongshu,
		status: LiveStatus.NotLive
	};
	// 使用路径参数的方式，取 livestream 字段的值
	let roomId = url.split('/livestream/')[1] || '';
	// 取 livestream 后面的数字，后面可能是 ? 或者 /，所以再次 split
	roomId = roomId.split('?')[0].split('/')[0] || '';
	let appApi = `https://www.xiaohongshu.com/api/sns/red/live/app/v1/ecology/outside/share_info?room_id=${roomId}`;
	try {
		let data = await invoke('request', { url: appApi, headers: getHeaders() });
		let json = JSON.parse(data as string);
		if (json.code != 0) {
			console.error('xiaohongshu api error', json);
			return info;
		}
		info.anchorName = json.data.host_info.nickname || '';
		info.anchorAvatar = json.data.host_info.avatar || '';
		info.roomCover = json.data.room.cover || '';
		info.viewerCount = json.data.room.member_count || '';
		info.title = json.data.room.name || '';
		let url = `http://live-play.xhscdn.com/live/${roomId}.flv`;
		info.streams.push({
			protocol: StreamingProtocol.Flv,
			url,
			resolution: 'default'
		});
		// 请求该 url，如果返回 404，则说明未直播
		let status = await invoke('try_request_get_status', { url, headers: getHeaders(), timeout: 1 });
		if ((status as number) === 200) {
			info.status = LiveStatus.Live;
		}
	} catch (e) {
		console.error('get live info for xiaohongshu failed: ', e);
		throw e;
	}
	return info;
}

function getHeaders() {
	return {
		Accept: 'application/json, text/plain, */*',
		'User-Agent':
			'Mozilla/5.0 (Windows NT 10.0; Win64; x64; rv:109.0) Gecko/20100101 Firefox/115.0',
		'Accept-Language': 'zh-CN,zh;q=0.8,zh-TW;q=0.7,zh-HK;q=0.5,en-US;q=0.3,en;q=0.2',
		Referer:
			'https://www.redelight.cn/hina/livestream/569077534207413574/1707413727088?share_source=&share_source_id=null&source=share_out_of_app&host_id=58bafe4282ec39085a56ece9&xhsshare=WeixinSession&appuid=5f3f478a00000000010005b3&apptime=1707413727'
	};
}
