mod api;
mod backstage;
mod ffmpeg;
mod kv;
mod manager;
mod model;
mod platform;
mod recorder;
mod request;
mod utils;
mod explorer;

#[tokio::main]
async fn main() {
    pretty_env_logger::init();
    kv::init().unwrap();
    backstage::init().await;
    api::listen(5577).await;
}
