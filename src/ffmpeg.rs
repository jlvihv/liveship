pub use anyhow::Result;
use log::{debug, info};
use std::process::{Child, Stdio};

pub fn record(ffmpeg_path: &str, url: &str, filename: &str) -> Result<Child> {
    info!("开始录制：{} -> {}", url, filename);
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
    // 调用 ffmpeg 命令
    let mut cmd = std::process::Command::new(ffmpeg_path);
    cmd.stdout(Stdio::piped()).stderr(Stdio::piped());
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
    debug!(
        "录制进程启动：{:?} for url: {}, to: {}",
        child.id(),
        url,
        filename
    );
    Ok(child)
}

/// 检查 ffmpeg 是否可用，返回 ffmpeg 版本号
pub fn check_ffmpeg(ffmpeg_path: &str) -> Result<String> {
    let output = std::process::Command::new(ffmpeg_path)
        .arg("-version")
        .output()?;
    let stdout = String::from_utf8_lossy(&output.stdout);
    Ok(stdout.to_string())
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
}
