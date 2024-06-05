import { LiveStatus, PlatformKind, StreamingProtocol, type LiveInfo, type Stream } from '@/model';

export function getLiveInfoForDouyu(url: string, html: string): LiveInfo {
	let info: LiveInfo = {
		url,
		title: '',
		anchorName: '',
		anchorAvatar: '',
		viewerCount: '',
		roomCover: '',
		streams: [],
		platformKind: PlatformKind.Douyu,
		status: LiveStatus.NotLive
	};
	// match <script id="SIGI_STATE" type="application/json">(.*?)</script><script id="SIGI_RETRY" type="application/json">
	let jsonStr = html.match(
		/<script id="SIGI_STATE" type="application\/json">(.*?)<\/script><script id="SIGI_RETRY" type="application\/json">/
	);
	if (!jsonStr || jsonStr.length < 2) {
		return info;
	}
	let json = JSON.parse(jsonStr[1]);
	console.log('douyu', json);

	return info;
}

export function getHeadersForDouyu() {
	return {
		'User-Agent':
			'Mozilla/5.0 (Windows NT 10.0; Win64; x64; rv:109.0) Gecko/20100101 Firefox/115.0',
		Referer:
			'https://m.douyu.com/3125893?rid=3125893&dyshid=0-96003918aa5365bc6dcb4933000316p1&dyshci=181',
		Cookie:
			'dy_did=413b835d2ae00270f0c69f6400031601; acf_did=413b835d2ae00270f0c69f6400031601; Hm_lvt_e99aee90ec1b2106afe7ec3b199020a7=1692068308,1694003758; m_did=96003918aa5365bc6dcb4933000316p1; dy_teen_mode=%7B%22uid%22%3A%22472647365%22%2C%22status%22%3A0%2C%22birthday%22%3A%22%22%2C%22password%22%3A%22%22%7D; PHPSESSID=td59qi2fu2gepngb8mlehbeme3; acf_auth=94fc9s%2FeNj%2BKlpU%2Br8tZC3Jo9sZ0wz9ClcHQ1akL2Nhb6ZyCmfjVWSlR3LFFPuePWHRAMo0dt9vPSCoezkFPOeNy4mYcdVOM1a8CbW0ZAee4ipyNB%2Bflr58; dy_auth=bec5yzM8bUFYe%2FnVAjmUAljyrsX%2FcwRW%2FyMHaoArYb5qi8FS9tWR%2B96iCzSnmAryLOjB3Qbeu%2BBD42clnI7CR9vNAo9mva5HyyL41HGsbksx1tEYFOEwxSI; wan_auth37wan=5fd69ed5b27fGM%2FGoswWwDo%2BL%2FRMtnEa4Ix9a%2FsH26qF0sR4iddKMqfnPIhgfHZUqkAk%2FA1d8TX%2B6F7SNp7l6buIxAVf3t9YxmSso8bvHY0%2Fa6RUiv8; acf_uid=472647365; acf_username=472647365; acf_nickname=%E7%94%A8%E6%88%B776576662; acf_own_room=0; acf_groupid=1; acf_phonestatus=1; acf_avatar=https%3A%2F%2Fapic.douyucdn.cn%2Fupload%2Favatar%2Fdefault%2F24_; acf_ct=0; acf_ltkid=25305099; acf_biz=1; acf_stk=90754f8ed18f0c24; Hm_lpvt_e99aee90ec1b2106afe7ec3b199020a7=1694003778'
	};
}
