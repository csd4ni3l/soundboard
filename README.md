# csd4ni3l Soundboard

Cross-platform soundboard made in Rust & Bevy. My first Rust project.
You might ask, why? And my answer is why not? Also because i wanted to learn Rust and this was a good way.

## Features & Requirements

| Topic | Linux | Windows | MacOS & Other
| -------- | ------- | ------- | ------- |
| Requirements | ALSA & PulseAudio/Pipewire-pulse, optionally FFmpeg for youtube downloader | Needs the [VB-Cable driver](https://vb-audio.com/Cable), optionally FFmpeg for youtube downloader | Unknown (optionally FFmpeg for youtube downloader)|
| Build Requirements | Rust, the `mold` linker and `clang` to compile fast | Rust, any C compiler | Unknown |
| FFmpeg | Optionally for youtube downloader | Optional, Automatic install on Windows 11 (winget) | Optionally for youtube downloader |
| Virtual Mic | Pulseaudio/Pipewire | VB-Cable | No |
| App Selection | Yes | No | No |
| Youtube Downloader support | Yes (ffmpeg required) | Yes (ffmpeg required) | Unknown (ffmpeg required) |
| Can others hear you? | Yes | Experimental | Unknown |
| Support | Best | Medium | None/Unknown |
| Download | [Download for Linux](https://github.com/csd4ni3l/soundboard/releases/download/latest/soundboard) | [Download for Windows](https://github.com/csd4ni3l/soundboard/releases/download/latest/soundboard.exe) | Build it yourself. | 