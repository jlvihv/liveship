import { LiveStatus, PlatformKind, StreamingProtocol, type LiveInfo, type Stream } from '@/model';
import { invoke } from '@tauri-apps/api/core';

export async function getLiveInfoForTiktok(url: string): Promise<LiveInfo> {
	let info: LiveInfo = {
		url,
		title: '',
		anchorName: '',
		anchorAvatar: '',
		viewerCount: '',
		roomCover: '',
		streams: [],
		platformKind: PlatformKind.Tiktok,
		status: LiveStatus.NotLive
	};
	try {
		// 首先请求页面内容
		let data = await invoke('request', { url, headers: getHeaders() });
		let html = data as string;
		// 解析 html，填充 LiveInfo
		parseHtmlAndFillLiveInfo(html, info);
	} catch (e) {
		console.error('get live info for tiktok failed: ', e);
		// 抛出一个错误
		throw e;
	}
	return info;
}

// 解析 html，填充 LiveInfo
function parseHtmlAndFillLiveInfo(html: string, info: LiveInfo) {
	// match <script id="SIGI_STATE" type="application/json">(.*?)</script><script id="SIGI_RETRY" type="application/json">
	let jsonStr = html.match(
		/<script id="SIGI_STATE" type="application\/json">(.*?)<\/script><script id="SIGI_RETRY" type="application\/json">/
	);
	if (!jsonStr || jsonStr.length < 2) {
		throw new Error('can not match json string');
	}
	let json = JSON.parse(jsonStr[1]);
	console.log('tiktok', json);
	let liveRoom = json.LiveRoom.liveRoomUserInfo;
	info.roomCover = liveRoom.liveRoom.coverUrl || '';
	info.viewerCount = liveRoom.liveRoom.liveRoomStats.userCount.toString() || '';
	info.title = liveRoom.liveRoom.title || '';
	let user = liveRoom.user;
	let nickname = user.nickname || '';
	let uniqueId = user.uniqueId || '';
	info.anchorName = `${nickname}(@${uniqueId})`;
	info.anchorAvatar = user.avatarThumb || '';
	let status = user.status || LiveStatus.NotLive;
	if (status !== 2) {
		return info;
	}
	info.status = LiveStatus.Live;
	let streamDataString: string = liveRoom.liveRoom.streamData.pull_data.stream_data;
	let streamDataJson = JSON.parse(streamDataString);
	let streamData: { [key: string]: any } = streamDataJson.data;
	// 遍历 streamData
	for (let key in streamData) {
		let stream = streamData[key];
		let flvUrl: string = stream.main.flv || '';
		if (flvUrl !== '') {
			info.streams.push({
				url: flvUrl.replace('https://', 'http://'),
				resolution: key || '',
				protocol: StreamingProtocol.Flv
			});
		}
		let hlsUrl: string = stream.main.hls || '';
		if (hlsUrl !== '') {
			info.streams.push({
				url: hlsUrl,
				resolution: key || '',
				protocol: StreamingProtocol.Hls
			});
		}
	}
}

function getHeaders() {
	return {
		'User-Agent':
			'Mozilla/5.0 (Windows NT 10.0; Win64; x64; rv:109.0) Gecko/20100101 Firefox/115.0',
		Accept: 'text/html,application/xhtml+xml,application/xml;q=0.9,image/avif,image/webp,*/*;q=0.8',
		Cookie:
			'ttwid=1%7CM-rF193sJugKuNz2RGNt-rh6pAAR9IMceUSzlDnPCNI%7C1683274418%7Cf726d4947f2fc37fecc7aeb0cdaee52892244d04efde6f8a8edd2bb168263269; tiktok_webapp_theme=light; tt_chain_token=VWkygAWDlm1cFg/k8whmOg==; passport_csrf_token=6e422c5a7991f8cec7033a8082921510; passport_csrf_token_default=6e422c5a7991f8cec7033a8082921510; d_ticket=f8c267d4af4523c97be1ccb355e9991e2ae06; odin_tt=320b5f386cdc23f347be018e588873db7f7aea4ea5d1813681c3fbc018ea025dde957b94f74146dbc0e3612426b865ccb95ec8abe4ee36cca65f15dbffec0deff7b0e69e8ea536d46e0f82a4fc37d211; cmpl_token=AgQQAPNSF-RO0rT04baWtZ0T_jUjl4fVP4PZYM2QPw; uid_tt=319b558dbba684bb1557206c92089cd113a875526a89aee30595925d804b81c7; uid_tt_ss=319b558dbba684bb1557206c92089cd113a875526a89aee30595925d804b81c7; sid_tt=ad5e736f4bedb2f6d42ccd849e706b1d; sessionid=ad5e736f4bedb2f6d42ccd849e706b1d; sessionid_ss=ad5e736f4bedb2f6d42ccd849e706b1d; store-idc=useast5; store-country-code=us; store-country-code-src=uid; tt-target-idc=useast5; tt-target-idc-sign=qXNk0bb1pDQ0FbCNF120Pl9WWMLZg9Edv5PkfyCbS4lIk5ieW5tfLP7XWROnN0mEaSlc5hg6Oji1pF-yz_3ZXnUiNMrA9wNMPvI6D9IFKKVmq555aQzwPIGHv0aQC5dNRgKo5Z5LBkgxUMWEojTKclq2_L8lBciw0IGdhFm_XyVJtbqbBKKgybGDLzK8ZyxF4Jl_cYRXaDlshZjc38JdS6wruDueRSHe7YvNbjxCnApEFUv-OwJANSPU_4rvcqpVhq3JI2VCCfw-cs_4MFIPCDOKisk5EhAo2JlHh3VF7_CLuv80FXg_7ZqQ2pJeMOog294rqxwbbQhl3ATvjQV_JsWyUsMd9zwqecpylrPvtySI2u1qfoggx1owLrrUynee1R48QlanLQnTNW_z1WpmZBgVJqgEGLwFoVOmRzJuFFNj8vIqdjM2nDSdWqX8_wX3wplohkzkPSFPfZgjzGnQX28krhgTytLt7BXYty5dpfGtsdb11WOFHM6MZ9R9uLVB; sid_guard=ad5e736f4bedb2f6d42ccd849e706b1d%7C1690990657%7C15525213%7CMon%2C+29-Jan-2024+08%3A11%3A10+GMT; sid_ucp_v1=1.0.0-KGM3YzgwYjZhODgyYWI1NjIwNTA0NjBmOWUxMGRhMjIzYTI2YjMxNDUKGAiqiJ30keKD5WQQwfCppgYYsws4AkDsBxAEGgd1c2Vhc3Q1IiBhZDVlNzM2ZjRiZWRiMmY2ZDQyY2NkODQ5ZTcwNmIxZA; ssid_ucp_v1=1.0.0-KGM3YzgwYjZhODgyYWI1NjIwNTA0NjBmOWUxMGRhMjIzYTI2YjMxNDUKGAiqiJ30keKD5WQQwfCppgYYsws4AkDsBxAEGgd1c2Vhc3Q1IiBhZDVlNzM2ZjRiZWRiMmY2ZDQyY2NkODQ5ZTcwNmIxZA; tt_csrf_token=dD0EIH8q-pe3qDQsCyyD1jLN6KizJDRjOEyk; __tea_cache_tokens_1988={%22_type_%22:%22default%22%2C%22user_unique_id%22:%227229608516049831425%22%2C%22timestamp%22:1683274422659}; ttwid=1%7CM-rF193sJugKuNz2RGNt-rh6pAAR9IMceUSzlDnPCNI%7C1694002151%7Cd89b77afc809b1a610661a9d1c2784d80ebef9efdd166f06de0d28e27f7e4efe; msToken=KfJAVZ7r9D_QVeQlYAUZzDFbc1Yx-nZz6GF33eOxgd8KlqvTg1lF9bMXW7gFV-qW4MCgUwnBIhbiwU9kdaSpgHJCk-PABsHCtTO5J3qC4oCTsrXQ1_E0XtbqiE4OVLZ_jdF1EYWgKNPT2SnwGkQ=; msToken=KfJAVZ7r9D_QVeQlYAUZzDFbc1Yx-nZz6GF33eOxgd8KlqvTg1lF9bMXW7gFV-qW4MCgUwnBIhbiwU9kdaSpgHJCk-PABsHCtTO5J3qC4oCTsrXQ1_E0XtbqiE4OVLZ_jdF1EYWgKNPT2SnwGkQ='
	};
}
