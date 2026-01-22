#!/bin/bash
pactl load-module module-null-sink sink_name=VirtualMic sink_properties=device.description="Virtual_Microphone"
pactl load-module module-remap-source master=VirtualMic.monitor source_name=VirtualMicSource source_properties=device.description="Virtual_Mic_Source"

PULSE_SINK=VirtualMic cargo run

pactl list modules short | grep "Virtual_Microphone" | cut -f1 | xargs -L1 pactl unload-module
pactl list modules short | grep "Virtual_Mic_Source" | cut -f1 | xargs -L1 pactl unload-module