#[cfg(target_os = "macos")]
use std::path::PathBuf;

#[cfg(not(target_os = "linux"))]
use std::process::Command;

pub fn open_in_folder(path: &str) -> Result<(), String> {
    // linux 直接返回错误
    #[cfg(target_os = "linux")]
    {
        Err(format!("Not implemented on Linux, can not open {}", path))
    }

    #[cfg(target_os = "windows")]
    {
        Command::new("explorer")
            .args(["/select,", &path]) // The comma after select is not a typo
            .spawn()
            .map_err(|e| e.to_string())?;
        Ok(())
    }

    #[cfg(target_os = "macos")]
    {
        let path_buf = PathBuf::from(&path);
        if path_buf.is_dir() {
            Command::new("open")
                .args([&path])
                .spawn()
                .map_err(|e| e.to_string())?;
        } else {
            Command::new("open")
                .args(["-R", &path])
                .spawn()
                .map_err(|e| e.to_string())?;
        }
        Ok(())
    }
}
