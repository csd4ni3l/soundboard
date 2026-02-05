
use std::process::Command;
use serde_json::Value;
use rodio::{OutputStream, OutputStreamBuilder, cpal::{self, traits::HostTrait}};

fn pactl_list(sink_type: &str) -> Value {
    let command_output = Command::new("pactl")
        .args(&["-f", "json", "list", sink_type])
        .output()
        .expect("Failed to execute process");

    if command_output.status.success() {
        serde_json::from_str(str::from_utf8(&command_output.stdout).expect("Failed to convert to string")).expect("Failed to parse sink JSON output")
    }
    else {
        Value::Null{}
    }
}

pub fn get_sink_by_index(sink_type: &str, index: String) -> Value {
    let sinks = pactl_list(sink_type);

    for sink in sinks.as_array().unwrap_or(&vec![]) {
        if sink["index"].as_u64().expect("sink index is not a number").to_string() == index {
            return sink.clone();
        }
    }

    return Value::Null{};
}

fn find_soundboard_sinks() -> Vec<Value> {
    let sink_inputs = pactl_list("sink-inputs");
    sink_inputs.as_array()
               .unwrap_or(&vec![])
               .iter()
               .filter(|sink| {sink["properties"]["node.name"] == "alsa_playback.soundboard"})
               .cloned()
               .collect()
}  

pub fn move_playback_to_sink() {
    let soundboard_sinks = find_soundboard_sinks();
    for sink in soundboard_sinks {
        let index = sink["index"].as_u64().expect("sink index is not a number").to_string();
        Command::new("pactl")
                    .args(&["move-sink-input", index.as_str(), "SoundboardSink"]) // as_str is needed here as you cannot instantly dereference a growing String (Rust...)
                    .output()
                    .expect("Failed to execute process");
    }
}

pub fn list_outputs() -> Vec<(String, String)> {
    let source_outputs = pactl_list("source-outputs");
    return source_outputs.as_array().unwrap_or(&vec![]).iter().filter_map(|sink| {
        let app_name = sink["properties"]["application.name"].as_str()?;
        let binary = sink["properties"]["application.process.binary"].as_str().unwrap_or("Unknown");
        let index = sink["index"].as_u64().expect("sink index is not a number").to_string();
        Some((format!("{} ({})", app_name, binary), index))
    }).collect();
}

pub fn move_index_to_virtualmic(index: String) {
    Command::new("pactl")
    .args(&["move-source-output", index.as_str(), "VirtualMicSource"]) // as_str is needed here as you cannot instantly dereference a growing String (Rust...)
    .output()
    .expect("Failed to execute process");
}

pub fn create_virtual_mic_linux() -> OutputStream {
    Command::new("pactl")
        .args(&["load-module", "module-null-sink", "sink_name=SoundboardSink", "sink_properties=device.description=\"Soundboard_Audio\""])
        .output()
        .expect("Failed to create SoundboardSink");
    
    Command::new("pactl")
        .args(&["load-module", "module-null-sink", "sink_name=VirtualMic", "sink_properties=device.description=\"Virtual_Microphone\""])
        .output()
        .expect("Failed to create VirtualMic");
    
    Command::new("pactl")
        .args(&["load-module", "module-remap-source", "master=VirtualMic.monitor", "source_name=VirtualMicSource", "source_properties=device.description=\"Virtual_Mic_Source\""])
        .output()
        .expect("Failed to create VirtualMicSource");

    // Soundboard audio -> speakers
    Command::new("pactl")
        .args(&["load-module", "module-loopback", "source=SoundboardSink.monitor", "sink=@DEFAULT_SINK@", "latency_msec=1"])
        .output()
        .expect("Failed to create soundboard to speakers loopback");
    
    // Soundboard audio -> VirtualMic
    Command::new("pactl")
        .args(&["load-module", "module-loopback", "source=SoundboardSink.monitor", "sink=VirtualMic", "latency_msec=1"])
        .output()
        .expect("Failed to create soundboard to VirtualMic loopback");
    
    // Microphone -> VirtualMic ONLY
    Command::new("pactl")
        .args(&["load-module", "module-loopback", "source=@DEFAULT_SOURCE@", "sink=VirtualMic", "latency_msec=1"])
        .output()
        .expect("Failed to create microphone loopback");

    Command::new("pactl")
        .args(&["set-sink-volume", "VirtualMic", "100%"])
        .output()
        .expect("Failed to set volume");
    
    Command::new("pactl")
        .args(&["set-sink-volume", "SoundboardSink", "100%"])
        .output()
        .expect("Failed to set soundboard volume");
    
    let host = cpal::host_from_id(cpal::HostId::Alsa).expect("Could not initialize ALSA");
    let device = host.default_output_device().expect("Could not get default output device");

    let stream = OutputStreamBuilder::from_device(device)
        .expect("Unable to open VirtualMic")
        .open_stream()
        .expect("Failed to open stream");

    move_playback_to_sink();

    return stream;
}

pub fn reload_sound() {
    let script = r#"
        pactl list modules short | grep "module-loopback" | cut -f1 | xargs -L1 pactl unload-module
        pactl list modules short | grep "Virtual_Microphone" | cut -f1 | xargs -L1 pactl unload-module
        pactl list modules short | grep "Virtual_Mic_Source" | cut -f1 | xargs -L1 pactl unload-module
        pactl list modules short | grep "Soundboard_Audio" | cut -f1 | xargs -L1 pactl unload-module
    "#;

    let output = Command::new("sh")
        .arg("-c")
        .arg(script)
        .output()
        .expect("Failed to execute process");

    if output.status.success() {
        println!("Modules unloaded successfully.");
    } else {
        println!("Error: {}", String::from_utf8_lossy(&output.stderr));
    }
}