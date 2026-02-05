Soundboard made in Rust & Bevy. My first Rust project.

# Support & Requirements

## Linux
- Needs the `mold` linker and `clang` to compile fast
- ALSA & PulseAudio/Pipewire-pulse is a requirement
- Can use auto-selection of app to use the virtual mic in.
- Auto-routes mic to virtual mic by default, so others can also hear you.

## Windows
- Needs the VB-Cable driver (https://vb-audio.com/Cable/)
- You need to still select the device inside the app you want to use it in.
- They only hear the soundboard as of right now, not your actual mic.

## MacOS & Other
- Might work as a music player with the default output device.
- Not supported and not planned.