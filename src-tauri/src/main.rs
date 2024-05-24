// Prevents additional console window on Windows in release, DO NOT REMOVE!!
#![cfg_attr(not(debug_assertions), windows_subsystem = "windows")]

use liveship_lib::{backstage, kv};

fn main() {
    let _ = kv::init();
    backstage::init_with_new_thread();
    liveship_lib::run();
}
