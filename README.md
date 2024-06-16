[中文](README.zh.md)

## Introduction

liveship is a compact and easy-to-use live streaming recording tool. It currently supports YouTube, TikTok, Twitch, Douyin, Huya, and Xiaohongshu, with plans to add support for more platforms in the future.

## How It Works

liveship essentially acts as a wrapper for ffmpeg. It simulates requests to obtain live stream URLs and then uses ffmpeg to record them. Therefore, you must have ffmpeg installed on your computer and specify the path to ffmpeg on the "Program Settings" page.

## Tech Stack

Proudly built with Rust, Tauri, and Svelte 5.

## Usage

liveship is a Tauri-based desktop application. You can download the binary files for your platform from the [release](https://github.com/jlvihv/liveship/releases/) page, then install and run it.

## FAQ

1. macOS prompts "The file is damaged and cannot be opened": This is because macOS restricts applications not from the App Store. You can remove the restriction by running the command `sudo xattr -d com.apple.quarantine /Applications/liveship.app` in the terminal.

## Special Thanks

The code for live stream parsing largely references the [DouyinLiveRecorder](https://github.com/ihmily/DouyinLiveRecorder) project. We extend our sincere thanks for their work.

## Special Note

liveship plans to introduce paid features after version 1.0 as part of my journey as an independent developer. However, all features will be open-source and free before version 1.0. Your suggestions and feedback are highly appreciated.

## License

CC BY-NC (Creative Commons Attribution-NonCommercial):

Allows copying, distribution, display, and performance of the work and its derivative works, but only for non-commercial purposes.
