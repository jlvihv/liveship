use tauri::{AppHandle, Manager};

pub mod backstage;
mod ffmpeg;
pub mod kv;
mod manager;
mod model;
mod platform;
mod recorder;
mod request;
mod utils;

#[cfg_attr(mobile, tauri::mobile_entry_point)]
pub fn run() {
    tauri::Builder::default()
        .setup(|app| {
            let main_window = app.get_webview_window("main").unwrap();
            main_window.on_window_event(|event| match event {
                tauri::WindowEvent::CloseRequested { .. } => {
                    // 程序退出前，停止所有录制，并更新数据库
                    manager::record::stop_all_record();
                }
                _ => {}
            });
            Ok(())
        })
        .plugin(tauri_plugin_shell::init())
        .plugin(tauri_plugin_single_instance::init(|app, _args, _cwd| {
            let _ = show_window(app);
        }))
        .invoke_handler(tauri::generate_handler![
            manager::record::start_record,
            manager::record::stop_record,
            manager::record::record_status,
            manager::history::get_all_history,
            manager::history::delete_history,
            manager::history::open_in_folder,
            manager::plan::get_all_plans,
            manager::plan::add_plan,
            manager::plan::delete_plan,
            manager::plan::update_plan_status,
            manager::plan::get_last_polling_time,
            manager::config::get_config,
            manager::config::set_config,
            manager::live::live_info,
            manager::ffmpeg_api::check_ffmpeg,
        ])
        .run(tauri::generate_context!())
        .expect("error while running tauri application");
}

fn show_window(app: &AppHandle) {
    let windows = app.webview_windows();

    windows
        .values()
        .next()
        .expect("Sorry, no window found")
        .set_focus()
        .expect("Can't Bring Window to Focus");
}
