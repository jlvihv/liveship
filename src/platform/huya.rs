use crate::{
    model::{LiveInfo, PlatformKind},
    recorder::Recorder,
    request,
};
use anyhow::{anyhow, Ok, Result};
use axum::{
    async_trait,
    http::{HeaderMap, HeaderValue},
};
use base64::prelude::*;

pub struct Huya;

impl Huya {
    pub fn new() -> Self {
        Self
    }
}

#[async_trait]
impl Recorder for Huya {
    fn kind(&self) -> PlatformKind {
        PlatformKind::Huya
    }

    async fn get_live_info(&self, url: &str) -> Result<LiveInfo> {
        let body = request::new_client_get_with_headers(url, Self::headers()?)
            .await?
            .text()
            .await?;
        println!("body: {:#?}", body);
        let re = regex::Regex::new(r#"stream: (\{"data".*?),"iWebDefaultBitRate""#)?;
        let json_str = re
            .captures(&body)
            .ok_or_else(|| anyhow!("stream not found"))?
            .get(1)
            .ok_or_else(|| anyhow!("stream not found"))?
            .as_str();
        let json_data: serde_json::Value = serde_json::from_str(&format!("{json_str}{}", "}"))?;
        // println!("json_data: {:#?}", json_data);
        let game_live_info = json_data
            .pointer("/data/0/gameLiveInfo")
            .ok_or_else(|| anyhow!("gameLiveInfo not found"))?
            .as_object()
            .ok_or_else(|| anyhow!("gameLiveInfo is not Object"))?;
        let game_stream_info_list = json_data
            .pointer("/data/0/gameStreamInfoList")
            .ok_or_else(|| anyhow!("gameStreamInfoList not found"))?
            .as_array()
            .ok_or_else(|| anyhow!("gameStreamInfoList is not Array"))?;
        let anchor_name = game_live_info
            .get("nick")
            .ok_or_else(|| anyhow!("nick not found"))?
            .as_str()
            .ok_or_else(|| anyhow!("nick is not String"))?;
        let select_cdn = game_stream_info_list
            .get(0)
            .ok_or_else(|| anyhow!("gameStreamInfoList is empty"))?;
        let flv_url = select_cdn
            .get("sFlvUrl")
            .ok_or_else(|| anyhow!("sFlvUrl not found"))?
            .as_str()
            .ok_or_else(|| anyhow!("sFlvUrl is not String"))?;
        let stream_name = select_cdn
            .get("sStreamName")
            .ok_or_else(|| anyhow!("sStreamName not found"))?
            .as_str()
            .ok_or_else(|| anyhow!("sStreamName is not String"))?;
        let flv_url_suffix = select_cdn
            .get("sFlvUrlSuffix")
            .ok_or_else(|| anyhow!("sFlvUrlShttps://www.huya.com/859042uffix not found"))?
            .as_str()
            .ok_or_else(|| anyhow!("sFlvUrlSuffix is not String"))?;
        let hls_url = select_cdn
            .get("sHlsUrl")
            .ok_or_else(|| anyhow!("sHlsUrl not found"))?
            .as_str()
            .ok_or_else(|| anyhow!("sHlsUrl is not String"))?;
        let hls_url_suffix = select_cdn
            .get("sHlsUrlSuffix")
            .ok_or_else(|| anyhow!("sHlsUrlSuffix not found"))?
            .as_str()
            .ok_or_else(|| anyhow!("sHlsUrlSuffix is not String"))?;
        let flv_anti_code = select_cdn
            .get("sFlvAntiCode")
            .ok_or_else(|| anyhow!("sFlvAntiCode not found"))?
            .as_str()
            .ok_or_else(|| anyhow!("sFlvAntiCode is not String"))?;
        let _hls_anti_code = select_cdn
            .get("sHlsAntiCode")
            .ok_or_else(|| anyhow!("sHlsAntiCode not found"))?
            .as_str()
            .ok_or_else(|| anyhow!("sHlsAntiCode is not String"))?;
        println!("flv_anti_code: {:#?}", flv_anti_code);
        let new_anti_code = Self::get_anti_code(flv_anti_code, stream_name)?;
        let flv_url = format!(
            "{}/{}.{}?{}&ratio=",
            flv_url, stream_name, flv_url_suffix, new_anti_code
        );
        let m3u8_url = format!(
            "{}/{}.{}?{}&ratio=",
            hls_url, stream_name, hls_url_suffix, new_anti_code
        );
        let quality_list = flv_anti_code.split("&exsphd=").collect::<Vec<&str>>();
        // println!("quality_list: {:#?}", quality_list);
        if quality_list.len() > 1 {
            // let quality_list = quality_list[1].split(",").collect::<Vec<&str>>();
            // let quality_list = quality_list
            //     .iter()
            //     .map(|x| x.split("_").collect::<Vec<&str>>())
            //     .collect::<Vec<Vec<&str>>>();
            // let quality_list = quality_list.iter().map(|x| x[1]).collect::<Vec<&str>>();
        }
        let mut streams = vec![];
        streams.push(crate::model::Stream {
            url: flv_url,
            resolution: "default".into(),
            protocol: crate::model::StreamingProtocol::Flv,
        });
        streams.push(crate::model::Stream {
            url: m3u8_url,
            resolution: "default".into(),
            protocol: crate::model::StreamingProtocol::Hls,
        });
        let info = LiveInfo {
            url: url.into(),
            anchor_name: anchor_name.into(),
            anchor_avatar: "".to_string(),
            title: "".to_string(),
            status: crate::model::LiveStatus::Live,
            viewer_count: "".to_string(),
            room_cover: "".to_string(),
            streams,
        };
        Ok(info)
    }
}

impl Huya {
    fn headers() -> Result<HeaderMap> {
        let mut headers = HeaderMap::new();
        headers.insert(
            "User-Agent",
            HeaderValue::from_str(
                "Mozilla/5.0 (Windows NT 10.0; Win64; x64; rv:123.0) Gecko/20100101 Firefox/123.0",
            )?,
        );
        headers.insert("Accept", HeaderValue::from_str("text/html,application/xhtml+xml,application/xml;q=0.9,image/avif,image/webp,*/*;q=0.8")?);
        headers.insert(
            "Accept-Language",
            HeaderValue::from_str("zh-CN,zh;q=0.8,zh-TW;q=0.7,zh-HK;q=0.5,en-US;q=0.3,en;q=0.2")?,
        );
        headers.insert("Cookie", HeaderValue::from_str("huya_ua=webh5&0.1.0&websocket; game_did=zXyXVqV1NF4ZeNWg7QaOFbpIEWqcsrxkoVy; alphaValue=0.80; isInLiveRoom=; guid=0a7df378828609654d01a205a305fb52; __yamid_tt1=0.8936157401010706; __yamid_new=CA715E8BC9400001E5A313E028F618DE; udb_guiddata=4657813d32ce43d381ea8ff8d416a3c2; udb_deviceid=w_756598227007868928; sdid=0UnHUgv0_qmfD4KAKlwzhqQB32nywGZJYLZl_9RLv0Lbi5CGYYNiBGLrvNZVszz4FEo_unffNsxk9BdvXKO_PkvC5cOwCJ13goOiNYGClLirWVkn9LtfFJw_Qo4kgKr8OZHDqNnuwg612sGyflFn1draukOt03gk2m3pwGbiKsB143MJhMxcI458jIjiX0MYq; Hm_lvt_51700b6c722f5bb4cf39906a596ea41f=1708583696; SoundValue=0.50; sdidtest=0UnHUgv0_qmfD4KAKlwzhqQB32nywGZJYLZl_9RLv0Lbi5CGYYNiBGLrvNZVszz4FEo_unffNsxk9BdvXKO_PkvC5cOwCJ13goOiNYGClLirWVkn9LtfFJw_Qo4kgKr8OZHDqNnuwg612sGyflFn1draukOt03gk2m3pwGbiKsB143MJhMxcI458jIjiX0MYq; sdidshorttest=test; __yasmid=0.8936157401010706; _yasids=__rootsid^%^3DCAA3838C53600001F4EE863017406250; huyawap_rep_cnt=4; udb_passdata=3; huya_web_rep_cnt=89; huya_flash_rep_cnt=20; Hm_lpvt_51700b6c722f5bb4cf39906a596ea41f=1709548534; _rep_cnt=3; PHPSESSID=r0klm0vccf08q1das65bnd8co1; guid=0a7df378828609654d01a205a305fb52; huya_hd_rep_cnt=8")?);
        Ok(headers)
    }

    // 获取反盗链码
    fn get_anti_code(old_anti_code: &str, stream_name: &str) -> Result<String> {
        // 这个 js 中包含了反盗链的相关逻辑，可以参考 https://hd.huya.com/cdn_libs/mobile/hysdk-m-202402211431.js
        let params_t = 100;
        let sdk_version: i64 = 2403051612;
        // sdk_sid 是毫秒时间戳
        let sdk_sid = chrono::Utc::now().timestamp_millis();

        // 计算 uuid 和 uid 参数值
        // let mut rng = rand::thread_rng();
        // let init_uuid: i64 = rng.gen_range(0..4294967295); // 直接初始化

        // init_uuid 的计算方法：
        // 1. 取时间戳后 10 位
        // 2. 将其放大 1000 倍
        // 3. 加上一个随机数 (0..1000)
        // 4. 对 4294967295 取余，确保在 32 位整数范围内
        let init_uuid = sdk_sid % 4294967295;
        // let uid = rng.gen_range(1400000000000..1400009999999); // 经过测试 uid 也可以使用 init_uuid 代替
        let uid = init_uuid;
        let seq_id = uid + sdk_sid; // 移动端请求的直播流地址中包含 seqId 参数

        // 计算 ws_time 参数值 (16 进制) 可以是当前毫秒时间戳，当然也可以直接使用 url_query['wsTime'][0]
        // 原始最大误差不得慢 240000 毫秒
        // let target_unix_time = (sdk_sid + 110624) / 1000;
        // let ws_time = format!("{:x}", target_unix_time);

        // 使用 & 分割字符串，得到参数列表
        let params = old_anti_code.split('&').collect::<Vec<&str>>();

        // ws_time 取 wsTime 参数值
        let ws_time = params
            .iter()
            .find(|&&x| x.starts_with("wsTime="))
            .ok_or_else(|| anyhow!("wsTime not found"))?
            .replace("wsTime=", "");

        // fm 参数值是经过 url 编码然后 base64 编码得到的，解码结果类似 DWq8BcJ3h6DJt6TY_$0_$1_$2_$3
        // 先进行 url 解码，再进行 base64 解码
        let fm = params
            .iter()
            .find(|&&x| x.starts_with("fm="))
            .ok_or_else(|| anyhow!("fm not found"))?
            .replace("fm=", "");
        let fm = urlencoding::decode(&fm)?.to_string();
        let fm = String::from_utf8(BASE64_STANDARD.decode(fm.as_bytes())?)?;
        // 第一个下划线之前的部分
        let fm: String = fm
            .split('_')
            .collect::<Vec<&str>>()
            .get(0)
            .map(|&x| x.into())
            .unwrap_or_default();
        println!("fm: {:#?}", fm);

        let ctype = params
            .iter()
            .find(|&&x| x.starts_with("ctype="))
            .ok_or_else(|| anyhow!("ctype not found"))?
            .replace("ctype=", "");
        let ws_secret_hash = format!("{}|{}|{}", seq_id, ctype, params_t);
        // md5 加密
        let ws_secret_hash = format!("{:x}", md5::compute(ws_secret_hash));
        let ws_secret = format!(
            "{}_{}_{}_{}_{}",
            fm, uid, stream_name, ws_secret_hash, ws_time
        );
        let ws_secret_md5 = format!("{:x}", md5::compute(ws_secret));
        let fs = params
            .iter()
            .find(|&&x| x.starts_with("fs="))
            .ok_or_else(|| anyhow!("fs not found"))?
            .replace("fs=", "");
        let anti_code = format!("wsSecret={ws_secret_md5}&wsTime={ws_time}&seqid={seq_id}&ctype={ctype}&ver=1&fs={fs}&uuid={init_uuid}&u={uid}&t={params_t}&sv={sdk_version}&sdk_sid={sdk_sid}&codec=264");
        Ok(anti_code)
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[tokio::test]
    async fn test_get_live_info() {
        let huya = Huya;
        // let url = "https://www.huya.com/107222";
        // let url = "https://www.huya.com/28916544";
        // let url = "https://www.huya.com/89523";
        // let url = "https://www.huya.com/51818";
        let url = "https://www.huya.com/749536";
        let live_info = huya.get_live_info(url).await.unwrap();
        println!("{:#?}", live_info);
    }
}
