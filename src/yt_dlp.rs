use std::{env::current_dir, fs::{File, exists}, io, process::Command};
use reqwest;
use rfd::{MessageButtons, MessageDialog, MessageDialogResult};

pub fn get_yt_dlp_path() -> String {
    if cfg!(target_os = "windows"){
        current_dir().expect("Failed to get current working directory").join("bin").join("yt-dlp.exe").to_string_lossy().to_string()
    }
    else if cfg!(target_os = "macos"){
        current_dir().expect("Failed to get current working directory").join("bin").join("yt-dlp_macos").to_string_lossy().to_string()
    }
    else if cfg!(target_os = "linux"){
        current_dir().expect("Failed to get current working directory").join("bin").join("yt-dlp_linux").to_string_lossy().to_string()
    }
    else {
        "".to_string()
    }
}

pub fn check_and_download_yt_dlp() {
    let url: &str;

    if cfg!(target_os = "windows"){
        url = "https://github.com/yt-dlp/yt-dlp/releases/latest/download/yt-dlp.exe";
    }
    else if cfg!(target_os = "macos"){
        url = "https://github.com/yt-dlp/yt-dlp/releases/latest/download/yt-dlp_macos";
    }
    else if cfg!(target_os = "linux"){
        url = "https://github.com/yt-dlp/yt-dlp/releases/latest/download/yt-dlp_linux";   
    }
    else {
        return;
    }

    if exists(get_yt_dlp_path()).expect("Could not check existence of yt dlp executable.") {
        return;
    }
    
    let mut body = reqwest::blocking::get(url).expect("Could not download yt-dlp");
    let mut out = File::create(get_yt_dlp_path()).expect("failed to create file");
    io::copy(&mut body, &mut out).expect("failed to copy content");
}

pub fn check_ffmpeg() -> bool{
    return std::process::Command::new("ffmpeg").spawn().is_ok();
}

pub fn check_and_download_ffmpeg() {
    if check_ffmpeg() {
        return;
    }

    if cfg!(target_os = "windows"){
        let confirmed = MessageDialog::new()
            .set_title("FFmpeg Download Optional.")
            .set_description("The youtube downloader depends on FFmpeg for mp3 conversion. This app can auto-install FFmpeg with winget. Do you want to install FFmpeg?")
            .set_buttons(MessageButtons::YesNo)
            .show();

        if confirmed == MessageDialogResult::Ok {
            Command::new("winget")
                .args(&["install", "BtbN.FFmpeg.GPL.Shared.8.0", "--source winget", "--accept-source-agreements", "--accept-package-agreements"]) // as_str is needed here as you cannot instantly dereference a growing String (Rust...)
                .output()
                .expect("Failed to execute process");
        }
    }
    else {
        MessageDialog::new()
            .set_title("FFmpeg Download Optional.")
            .set_description("The youtube downloader depends on FFmpeg for mp3 conversion. You are on a Linux or Darwin based OS. If you want to use the Youtube Downloader, you need to install FFmpeg and libavcodec shared libraries from your package manager to make sure it is in PATH.")
            .set_buttons(MessageButtons::Ok)
            .show();
    }
}