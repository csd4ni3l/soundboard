cargo run

pactl list modules short | grep "module-loopback" | cut -f1 | xargs -L1 pactl unload-module
pactl list modules short | grep "Virtual_Microphone" | cut -f1 | xargs -L1 pactl unload-module
pactl list modules short | grep "Virtual_Mic_Source" | cut -f1 | xargs -L1 pactl unload-module
pactl list modules short | grep "Soundboard_Audio" | cut -f1 | xargs -L1 pactl unload-module