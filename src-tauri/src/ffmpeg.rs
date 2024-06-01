use anyhow::anyhow;
pub use anyhow::Result;
use ffmpeg_sidecar::download::{download_ffmpeg_package, ffmpeg_download_url, unpack_ffmpeg};
use std::process::{Child, Stdio};

use crate::config::config_dir;

pub fn record(ffmpeg_path: &str, url: &str, filename: &str) -> Result<Child> {
    println!("开始录制：{} -> {}", url, filename);

    // 调用 ffmpeg 命令
    let mut cmd = std::process::Command::new(ffmpeg_path);
    cmd.stdout(Stdio::piped()).stderr(Stdio::piped());
    let ffmpeg_command = build_ffmpeg_command(url, filename);
    cmd.args(&ffmpeg_command);
    let mut child = cmd.spawn()?;
    // 立刻 try_wait 一下，看是否有错误
    if let Some(status) = child.try_wait()? {
        let output = child.wait_with_output()?;
        let stdout = String::from_utf8_lossy(&output.stdout);
        let stderr = String::from_utf8_lossy(&output.stderr);
        let error_message = format!(
            "status: {:?}\nstdout: {}\nstderr: {}",
            status, stdout, stderr
        );
        return Err(anyhow::anyhow!(error_message));
    }
    println!(
        "录制进程启动：{:?} for url: {}, to: {}",
        child.id(),
        url,
        filename
    );
    Ok(child)
}

fn build_ffmpeg_command(url: &str, filename: &str) -> Vec<String> {
    let user_agent = r#""Mozilla/5.0 (X11; Linux x86_64) AppleWebKit/537.36 (KHTML, like Gecko) Chrome/124.0.0.0 Safari/537.36""#;
    let analyzeduration = "20000000";
    let probesize = "10000000";
    let bufsize = "8000k";
    let max_muxing_queue_size = "1024";

    let mut ffmpeg_command = vec![
        "-y",
        "-v",
        "verbose",
        "-rw_timeout",
        "30000000",
        "-loglevel",
        "error",
        "-hide_banner",
        "-user_agent",
        user_agent,
        "-protocol_whitelist",
        "rtmp,crypto,file,http,https,tcp,tls,udp,rtp",
        "-thread_queue_size",
        "1024",
        "-analyzeduration",
        analyzeduration,
        "-probesize",
        probesize,
        "-fflags",
        "+discardcorrupt",
        "-i",
        &url,
        "-bufsize",
        bufsize,
        "-sn",
        "-dn",
        "-reconnect_delay_max",
        "60",
        "-reconnect_streamed",
        "-reconnect_at_eof",
        "-max_muxing_queue_size",
        max_muxing_queue_size,
        "-correct_ts_overflow",
        "1",
    ];
    let push_command = [
        "-c:v", "copy", "-c:a", "copy", "-map", "0", "-f", "mpegts", filename,
    ];
    ffmpeg_command.extend_from_slice(&push_command);
    ffmpeg_command.into_iter().map(|s| s.into()).collect()
}

/// 自动下载对应平台的 ffmpeg
pub fn download_ffmpeg() -> Result<String> {
    let download_url = ffmpeg_download_url()?;
    let config_dir = config_dir()?;
    let archive_path = download_ffmpeg_package(download_url, &config_dir)?;
    // 解压
    unpack_ffmpeg(&archive_path, &config_dir)?;
    let ffmpeg_path = if cfg!(target_os = "windows") {
        config_dir.join("ffmpeg.exe")
    } else {
        config_dir.join("ffmpeg")
    };
    let ffmpeg_path = ffmpeg_path
        .to_str()
        .ok_or_else(|| anyhow!("can not convert PathBuf to String"))?
        .to_string();
    Ok(ffmpeg_path)
}

#[cfg(test)]
mod tests {
    use super::*;

    #[tokio::test]
    async fn test_record() {
        let url = "http://pull-hls-l13.douyincdn.com/stage/stream-691574246930121144_or4.m3u8?expire=1715939773&sign=f73837f8a9bac9cac894a331e8a621cf";
        let filename = "test.ts";
        let child = record("ffmpeg", url, filename).unwrap();
        let output = child.wait_with_output().unwrap();
        let stdout = String::from_utf8_lossy(&output.stdout);
        let stderr = String::from_utf8_lossy(&output.stderr);
        println!("stdout: {}", stdout);
        println!("stderr: {}", stderr);
    }

    #[test]
    fn test_download_ffmpeg() {
        let ffmpeg_path = download_ffmpeg().unwrap();
        println!("ffmpeg_path: {}", ffmpeg_path);
    }
}
