use anyhow::anyhow;
pub use anyhow::Result;
use ffmpeg_sidecar::download::{download_ffmpeg_package, ffmpeg_download_url, unpack_ffmpeg};
use std::process::{Child, Stdio};

use crate::{config::config_dir, model::RecordingOption};

/// 给定 ffmpeg 命令，这里只负责执行
pub fn execute_ffmpeg_command(ffmpeg_command: Vec<String>) -> Result<Child> {
    println!("ffmpeg_command: {:?}", ffmpeg_command);
    // 调用 ffmpeg 命令
    let mut cmd = std::process::Command::new("ffmpeg");
    // 特定于 windows 的实现，使用 CommandExt，避免出现黑窗口
    #[cfg(target_os = "windows")]
    {
        use std::os::windows::process::CommandExt;
        cmd.creation_flags(0x08000000); // CREATE_NO_WINDOW
    }
    // cmd.stdout(Stdio::piped()).stderr(Stdio::piped());
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
    println!("录制进程启动：{:?}", child.id());
    Ok(child)
}

/// 给定 ffmpeg 命令，执行并返回输出
pub fn execute_ffmpeg_command_return_output(ffmpeg_command: Vec<String>) -> Result<String> {
    println!("ffmpeg_command: {:?}", ffmpeg_command);
    // 调用 ffmpeg 命令
    let output = std::process::Command::new("ffmpeg")
        .args(&ffmpeg_command)
        .output()?;
    let stdout = String::from_utf8_lossy(&output.stdout);
    let stderr = String::from_utf8_lossy(&output.stderr);
    if !output.status.success() {
        return Err(anyhow::anyhow!(
            "status: {:?}\nstdout: {}\nstderr: {}",
            output.status,
            stdout,
            stderr
        ));
    }
    Ok(stdout.to_string())
}

pub fn record(
    ffmpeg_path: &str,
    url: &str,
    filename: &str,
    option: Option<RecordingOption>,
) -> Result<Child> {
    println!(
        "开始录制：{} -> {}, recording option: {:?}",
        url, filename, option
    );

    // 调用 ffmpeg 命令
    let mut cmd = std::process::Command::new(ffmpeg_path);
    // 特定于 windows 的实现，使用 CommandExt，避免出现黑窗口
    #[cfg(target_os = "windows")]
    {
        use std::os::windows::process::CommandExt;
        cmd.creation_flags(0x08000000); // CREATE_NO_WINDOW
    }
    cmd.stdout(Stdio::piped()).stderr(Stdio::piped());
    let ffmpeg_command = build_ffmpeg_record_command(
        url,
        filename,
        option.map(|o| o.use_proxy).unwrap_or_default(),
    );
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

fn build_ffmpeg_record_command(url: &str, filename: &str, proxy: Option<String>) -> Vec<String> {
    let user_agent = r#""Mozilla/5.0 (X11; Linux x86_64) AppleWebKit/537.36 (KHTML, like Gecko) Chrome/124.0.0.0 Safari/537.36""#;
    let analyzeduration = "20000000";
    let probesize = "10000000";
    let bufsize = "8000k";
    let max_muxing_queue_size = "1024";
    let mut ffmpeg_command = vec![];
    if let Some(proxy) = &proxy {
        ffmpeg_command.extend_from_slice(&["-http_proxy", proxy.as_str()] as &[&str]);
    }
    let record_command = vec![
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
        "rtmp,crypto,file,http,https,tcp,tls,udp,rtp,httpproxy",
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
    ffmpeg_command.extend_from_slice(&record_command);
    ffmpeg_command.extend_from_slice(&push_command);
    ffmpeg_command.into_iter().map(|s| s.into()).collect()
}

/// 使用 ffmpeg 转换 ts 为 mp4
#[allow(unused)]
pub fn convert_ts_to_mp4(ts_file: &str, delete_origin_file: bool) -> Result<()> {
    // 判断是否以 ts 结尾
    if !ts_file.ends_with(".ts") {
        return Err(anyhow!("file is not end with .ts"));
    }
    // 检查文件是否存在
    if !std::path::Path::new(ts_file).exists() {
        return Err(anyhow!("file not exists"));
    }
    // 替换文件后缀，生成 mp4 文件名，替换最后的 '.ts' 为 '.mp4'
    let mp4_file = ts_file.replace(".ts", ".mp4");
    let ffmpeg_command = vec![
        "-i",
        ts_file,
        "-c:v",
        "copy",
        "-c:a",
        "copy",
        "-f",
        "mp4",
        mp4_file.as_str(),
    ]
    .iter()
    .map(|s| s.to_string())
    .collect();
    execute_ffmpeg_command_return_output(ffmpeg_command)?;
    if delete_origin_file {
        std::fs::remove_file(ts_file)?;
    }
    Ok(())
}

/// 转换为 m4a
#[allow(unused)]
pub fn convert_ts_to_m4a(ts_file: &str, delete_origin_file: bool) -> Result<()> {
    // 判断是否以 ts 结尾
    if !ts_file.ends_with(".ts") {
        return Err(anyhow!("file is not end with .ts"));
    }
    // 检查文件是否存在
    if !std::path::Path::new(ts_file).exists() {
        return Err(anyhow!("file not exists"));
    }
    // 替换文件后缀，生成 m4a 文件名，替换最后的 '.ts' 为 '.m4a'
    let m4a_file = ts_file.replace(".ts", ".m4a");
    let ffmpeg_command = vec![
        "-i",
        ts_file,
        "-n",
        "-vn",
        "-c:a",
        "aac",
        "-bsf:a",
        "aac_adtstoasc",
        "-ab",
        "320k",
        m4a_file.as_str(),
    ]
    .iter()
    .map(|s| s.to_string())
    .collect();
    execute_ffmpeg_command_return_output(ffmpeg_command)?;
    if delete_origin_file {
        std::fs::remove_file(ts_file)?;
    }
    Ok(())
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
        let child = record("ffmpeg", url, filename, None).unwrap();
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
