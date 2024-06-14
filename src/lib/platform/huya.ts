import { LiveStatus, PlatformKind, StreamingProtocol, type LiveInfo } from '@/model';
import CryptoJS from 'crypto-js';
import { fetch } from '@tauri-apps/plugin-http';

export async function getLiveInfoForHuya(url: string): Promise<LiveInfo> {
	let info: LiveInfo = {
		url,
		title: '',
		anchorName: '',
		anchorAvatar: '',
		viewerCount: '',
		roomCover: '',
		streams: [],
		platformKind: PlatformKind.Huya,
		status: LiveStatus.NotLive
	};

	try {
		// 首先请求页面内容
		let resp = await fetch(url, {
			method: 'GET',
			headers: getHeaders()
		});
		let html = await resp.text();
		// 解析 html，填充 LiveInfo
		parseHtmlAndFillLiveInfo(html, info);
	} catch (e) {
		console.error('get live info for huya failed: ', e);
		// 抛出一个错误
		throw e;
	}
	return info;
}

// 解析 html，填充 LiveInfo
function parseHtmlAndFillLiveInfo(html: string, info: LiveInfo) {
	// match stream: (\{"data".*?),"iWebDefaultBitRate"
	let jsonStr = html.match(/stream: (\{"data".*?),"iWebDefaultBitRate"/);
	if (!jsonStr || jsonStr.length < 2) {
		return info;
	}
	let json = JSON.parse(jsonStr[1] + '}');
	let gameLiveInfo = json.data[0].gameLiveInfo;
	let gameStreamInfoList = json.data[0].gameStreamInfoList;
	info.anchorName = gameLiveInfo.nick || '';
	info.anchorAvatar = gameLiveInfo.avatar180 || '';
	info.title = gameLiveInfo.introduction || '';
	info.roomCover = gameLiveInfo.screenshot || '';
	let selectCdn = gameStreamInfoList[0];
	let flvUrl = selectCdn.sFlvUrl || '';
	let streamName = selectCdn.sStreamName || '';
	let flvUrlSuffix = selectCdn.sFlvUrlSuffix || '';
	let flvAntiCode = selectCdn.sFlvAntiCode || '';
	let newAntiCode = getAntiCode(flvAntiCode, streamName);
	if (!newAntiCode) {
		return info;
	}
	let flvUrlWithAntiCode = `${flvUrl}/${streamName}.${flvUrlSuffix}?${newAntiCode}&ratio=`;
	let qualityList: Array<any> = flvAntiCode.split('&exsphd=') || [];
	if (qualityList.length > 1) {
		console.log('qualityList', qualityList);
	}
	info.streams.push({
		url: flvUrlWithAntiCode,
		resolution: 'default',
		protocol: StreamingProtocol.Flv
	});
	info.status = LiveStatus.Live;
}

function getAntiCode(oldAntiCode: string, streamName: string): string | undefined {
	if (oldAntiCode === '') {
		return;
	}
	let paramsT = 100;
	let sdkVersion = 2403051612;
	// sdk_sid 是毫秒时间戳
	let sdkSid = Date.now();

	// init_uuid 的计算方法：
	// 1. 取时间戳后 10 位
	// 2. 将其放大 1000 倍
	// 3. 加上一个随机数 (0..1000)
	// 4. 对 4294967295 取余，确保在 32 位整数范围内
	//     let init_uuid = sdk_sid % 4294967295;
	let initUuid = sdkSid % 4294967295;
	let uid = initUuid;
	let seqId = uid + sdkSid; // 移动端请求的直播流地址中包含 seqId 参数

	// 使用 & 分割字符串，得到参数列表
	let params = oldAntiCode.split('&');

	// ws_time 取 wsTime 参数值
	let wsTime = params.find((x) => x.startsWith('wsTime='))?.replace('wsTime=', '') || '';

	// fm 参数值是经过 url 编码然后 base64 编码得到的，解码结果类似 DWq8BcJ3h6DJt6TY_$0_$1_$2_$3
	let fm = params.find((x) => x.startsWith('fm='))?.replace('fm=', '') || '';
	// 先进行 url 解码，再进行 base64 解码
	fm = decodeURIComponent(fm);
	fm = atob(fm);
	// 第一个下划线之前的部分
	fm = fm.split('_')[0] || '';

	let ctype = params.find((x) => x.startsWith('ctype='))?.replace('ctype=', '') || '';
	let wsSecretHash = `${seqId}|${ctype}|${paramsT}`;
	// md5 加密
	wsSecretHash = CryptoJS.MD5(wsSecretHash).toString();
	let wsSecret = `${fm}_${uid}_${streamName}_${wsSecretHash}_${wsTime}`;
	let wsSecretMd5 = CryptoJS.MD5(wsSecret).toString();
	let fs = params.find((x) => x.startsWith('fs='))?.replace('fs=', '') || '';
	let antiCode = `wsSecret=${wsSecretMd5}&wsTime=${wsTime}&seqid=${seqId}&ctype=${ctype}&ver=1&fs=${fs}&uuid=${initUuid}&u=${uid}&t=${paramsT}&sv=${sdkVersion}&sdk_sid=${sdkSid}&codec=264`;
	return antiCode;
}

function getHeaders() {
	return {
		'User-Agent':
			'Mozilla/5.0 (Windows NT 10.0; Win64; x64; rv:109.0) Gecko/20100101 Firefox/115.0',
		Accept: 'text/html,application/xhtml+xml,application/xml;q=0.9,image/avif,image/webp,*/*;q=0.8',
		'Accept-Language': 'zh-CN,zh;q=0.8,zh-TW;q=0.7,zh-HK;q=0.5,en-US;q=0.3,en;q=0.2',
		Cookie:
			'huya_ua=webh5&0.1.0&websocket; game_did=zXyXVqV1NF4ZeNWg7QaOFbpIEWqcsrxkoVy; alphaValue=0.80; isInLiveRoom=; guid=0a7df378828609654d01a205a305fb52; __yamid_tt1=0.8936157401010706; __yamid_new=CA715E8BC9400001E5A313E028F618DE; udb_guiddata=4657813d32ce43d381ea8ff8d416a3c2; udb_deviceid=w_756598227007868928; sdid=0UnHUgv0_qmfD4KAKlwzhqQB32nywGZJYLZl_9RLv0Lbi5CGYYNiBGLrvNZVszz4FEo_unffNsxk9BdvXKO_PkvC5cOwCJ13goOiNYGClLirWVkn9LtfFJw_Qo4kgKr8OZHDqNnuwg612sGyflFn1draukOt03gk2m3pwGbiKsB143MJhMxcI458jIjiX0MYq; Hm_lvt_51700b6c722f5bb4cf39906a596ea41f=1708583696; SoundValue=0.50; sdidtest=0UnHUgv0_qmfD4KAKlwzhqQB32nywGZJYLZl_9RLv0Lbi5CGYYNiBGLrvNZVszz4FEo_unffNsxk9BdvXKO_PkvC5cOwCJ13goOiNYGClLirWVkn9LtfFJw_Qo4kgKr8OZHDqNnuwg612sGyflFn1draukOt03gk2m3pwGbiKsB143MJhMxcI458jIjiX0MYq; sdidshorttest=test; __yasmid=0.8936157401010706; _yasids=__rootsid^%^3DCAA3838C53600001F4EE863017406250; huyawap_rep_cnt=4; udb_passdata=3; huya_web_rep_cnt=89; huya_flash_rep_cnt=20; Hm_lpvt_51700b6c722f5bb4cf39906a596ea41f=1709548534; _rep_cnt=3; PHPSESSID=r0klm0vccf08q1das65bnd8co1; guid=0a7df378828609654d01a205a305fb52; huya_hd_rep_cnt=8'
	};
}
