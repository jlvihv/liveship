import { LiveStatus, PlatformKind, StreamingProtocol, type LiveInfo, type Stream } from '@/model';

export function getLiveInfoForXiaohongshu(url: string, html: string): LiveInfo {
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
	// 取 appuid 字段的值
	let appuid = url.split('appuid=')[1].split('&')[0] || '';
	// 使用路径参数的方式，取 livestream 字段的值
	let roomId = url.split('/livestream/')[1] || '';
	// 取 livestream 后面的数字，后面可能是 ? 或者 /，所以再次 split
	roomId = roomId.split('?')[0].split('/')[0] || '';
	// let app_api = format!("https://www.xiaohongshu.com/api/sns/red/live/app/v1/ecology/outside/share_info?room_id={}", room_id);
	let appApi = `https://www.xiaohongshu.com/api/sns/red/live/app/v1/ecology/outside/share_info?room_id=${roomId}`;
	// let resp = request::get_with_headers(&app_api, Self::headers()?).await?;
	// let body: JsonValue = serde_json::from_slice(&resp.bytes().await?)?;
	// let anchor_name = body
	//     .pointer("/data/host_info/nickname")
	//     .unwrap_or(&JsonValue::Null)
	//     .as_str()
	//     .unwrap_or_default();
	// let anchor_avatar = body
	//     .pointer("/data/host_info/avatar")
	//     .unwrap_or(&JsonValue::Null)
	//     .as_str()
	//     .unwrap_or_default()
	//     .split("?")
	//     .collect::<Vec<&str>>()[0];
	// let room_cover = body
	//     .pointer("/data/room/cover")
	//     .unwrap_or(&JsonValue::Null)
	//     .as_str()
	//     .unwrap_or_default()
	//     .split("?")
	//     .collect::<Vec<&str>>()[0];
	// let room_title = body
	//     .pointer("/data/room/name")
	//     .unwrap_or(&JsonValue::Null)
	//     .as_str()
	//     .unwrap_or_default();
	// let flv_url = format!(
	//     "http://live-play.xhscdn.com/live/{}.flv?uid={}",
	//     room_id, appuid
	// );

	return info;
}

export function getHeadersForXiaohongshu() {
	return {
		Accept: 'application/json, text/plain, */*',
		'User-Agent':
			'Mozilla/5.0 (Windows NT 10.0; Win64; x64; rv:109.0) Gecko/20100101 Firefox/115.0',
		'Accept-Language': 'zh-CN,zh;q=0.8,zh-TW;q=0.7,zh-HK;q=0.5,en-US;q=0.3,en;q=0.2',
		Referer:
			'https://www.redelight.cn/hina/livestream/569077534207413574/1707413727088?share_source=&share_source_id=null&source=share_out_of_app&host_id=58bafe4282ec39085a56ece9&xhsshare=WeixinSession&appuid=5f3f478a00000000010005b3&apptime=1707413727'
	};
}
