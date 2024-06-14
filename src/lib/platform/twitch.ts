import { LiveStatus, PlatformKind, StreamingProtocol, type LiveInfo, type Stream } from '@/model';
import { fetch } from '@tauri-apps/plugin-http';

export async function getLiveInfoForTwitch(url: string): Promise<LiveInfo> {
	let info: LiveInfo = {
		url,
		title: '',
		anchorName: '',
		anchorAvatar: '',
		viewerCount: '',
		roomCover: '',
		streams: [],
		platformKind: PlatformKind.Twitch,
		status: LiveStatus.NotLive
	};
	// 先找出 uid, 从 url 中提取，例如 https://www.twitch.tv/uid
	let uid = url.split('/').pop();
	try {
		let resp = await fetch('https://gql.twitch.tv/gql', {
			method: 'POST',
			headers: getHeadersForStream(),
			body: JSON.stringify({
				operationName: 'PlaybackAccessToken_Template',
				query:
					'query PlaybackAccessToken_Template($login: String!, $isLive: Boolean!, $vodID: ID!, $isVod: Boolean!, $playerType: String!) {  streamPlaybackAccessToken(channelName: $login, params: {platform: "web", playerBackend: "mediaplayer", playerType: $playerType}) @include(if: $isLive) {    value    signature   authorization { isForbidden forbiddenReasonCode }   __typename  }  videoPlaybackAccessToken(id: $vodID, params: {platform: "web", playerBackend: "mediaplayer", playerType: $playerType}) @include(if: $isVod) {    value    signature   __typename  }}',
				variables: {
					isLive: true,
					login: uid,
					isVod: false,
					vodID: '',
					playerType: 'site'
				}
			})
		});
		let jsonData1 = JSON.parse(await resp.text());
		let token = jsonData1.data.streamPlaybackAccessToken.value;
		let sign = jsonData1.data.streamPlaybackAccessToken.signature;
		let resp2 = await fetch('https://gql.twitch.tv/gql', {
			method: 'POST',
			headers: getHeaders(token),
			body: JSON.stringify([
				{
					operationName: 'ChannelShell',
					variables: {
						login: uid
					},
					extensions: {
						persistedQuery: {
							version: 1,
							sha256Hash: '580ab410bcd0c1ad194224957ae2241e5d252b2c5173d8e0cce9d32d5bb14efe'
						}
					}
				}
			])
		});
		let json = JSON.parse(await resp2.text());
		await parseJsonAndFillLiveInfo(json, info, sign, token);
	} catch (e) {
		console.error('get live info for twitch failed: ', e);
		throw e;
	}
	return info;
}

function getHeaders(token: string) {
	return {
		'User-Agent':
			'Mozilla/5.0 (Windows NT 10.0; Win64; x64; rv:124.0) Gecko/20100101 Firefox/124.0',
		'Accept-Language': 'zh-CN',
		Referer: 'https://www.twitch.tv/',
		'Client-Id': 'kimne78kx3ncx6brgo4mv6wki5h1ko',
		'Client-Integrity': token,
		'Content-Type': 'text/plain;charset=UTF-8'
	};
}

function getHeadersForStream() {
	return {
		'User-Agent':
			'Mozilla/5.0 (Windows NT 10.0; Win64; x64) AppleWebKit/537.36 (KHTML, like Gecko) Chrome/124.0.0.0 Safari/537.36',
		'Accept-Language': 'en-US',
		Referer: 'https://www.twitch.tv/',
		'Client-ID': 'kimne78kx3ncx6brgo4mv6wki5h1ko'
	};
}

async function parseJsonAndFillLiveInfo(json: any, info: LiveInfo, sign: string, token: string) {
	let userData = json[0].data.userOrError;
	let loginName = userData.login;
	info.anchorName = `${userData.displayName}-${loginName}`;
	info.anchorAvatar = userData.profileImageURL;
	info.title = info.anchorName; // 找不到标题，就用主播名代替吧
	// 如果有 userData.stream 字段，说明正在直播，否则不在直播
	if (userData.stream) {
		info.status = LiveStatus.Live;
		info.viewerCount = userData.stream.viewersCount.toString() || '';
		let playSessionIds = ['bdd22331a986c7f1073628f2fc5b19da', '064bc3ff1722b6f53b0b5b8c01e46ca5'];
		let playSessionId = playSessionIds[Math.floor(Math.random() * playSessionIds.length)];
		let params = {
			acmb: 'e30=',
			allow_sourc: 'true',
			browser_family: 'firefox',
			browser_version: '124.0',
			cdm: 'wv',
			fast_bread: 'true',
			os_name: 'Windows',
			os_version: 'NT%2010.0',
			p: '3553732',
			platform: 'web',
			play_session_id: playSessionId,
			player_backend: 'mediaplayer',
			player_version: '1.28.0-rc.1',
			playlist_include_framerate: 'true',
			reassignments_supported: 'true',
			sig: sign,
			token: token,
			transcode_mode: 'cbr_v1'
		};
		let access_key = new URLSearchParams(params).toString();
		let m3u8_url = `https://usher.ttvnw.net/api/channel/hls/${loginName}.m3u8?${access_key}`;
		info.streams.push({
			url: m3u8_url,
			resolution: 'default',
			protocol: StreamingProtocol.Hls
		});
		let playUrlList = await getPlayUrlList(m3u8_url, getHeadersForStream());
		// 遍历播放地址列表，添加到 info 中
		for (let i of playUrlList) {
			info.streams.push({
				url: i.url,
				resolution: i.groupId,
				protocol: StreamingProtocol.Hls
			});
		}
	} else {
		info.status = LiveStatus.NotLive;
	}
}

async function getPlayUrlList(m3u8: string, headers: any) {
	try {
		let resp = await fetch(m3u8, {
			method: 'GET',
			headers
		});
		let playUrlList = [];
		// 找 GROUP-ID= 字样，这是分辨率
		let groupPattern = /GROUP-ID="([^"]+)"/;
		let groupId: string = '';
		for (let line of (await resp.text()).split('\n')) {
			if (groupPattern.test(line)) {
				groupId = line.match(groupPattern)![0];
			}

			if (line.startsWith('https://')) {
				// groupId 去掉 GROUP-ID=" 和 "，只保留分辨率
				groupId = groupId.replace('GROUP-ID="', '').replace('"', '');
				playUrlList.push({ groupId: groupId, url: line.trim() });
			}
		}
		return playUrlList;
	} catch (e) {
		console.error('get play url list failed: ', e);
		throw e;
	}
}
