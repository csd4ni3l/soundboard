
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

pub fn get_device_by_index(sink_type: &str, index: String) -> Value {
    let devices = pactl_list(sink_type);

    for device in devices.as_array().unwrap_or(&vec![]) {
        if device["index"].as_u64().expect("Device index is not a number").to_string() == index {
            return device.clone();
        }
    }

    return Value::Null{};
}

pub fn move_playback_to_sink() {
    let sink_inputs = pactl_list("sink-inputs");
    for device in sink_inputs.as_array().unwrap_or(&vec![]) {
        if device["properties"]["node.name"] == "alsa_playback.soundboard" {
            let index = device["index"].as_u64().expect("Device index is not a number").to_string();
            Command::new("pactl")
            .args(&["move-sink-input", index.as_str(), "VirtualMic"]) // as_str is needed here as you cannot instantly dereference a growing String (Rust...)
            .output()
            .expect("Failed to execute process");
        }
    }
}

pub fn list_outputs() -> Vec<(String, String)> {
    let source_outputs = pactl_list("source-outputs");
    return source_outputs.as_array().unwrap_or(&vec![]).iter().filter_map(|device| {
        let app_name = device["properties"]["application.name"].as_str()?;
        let binary = device["properties"]["application.process.binary"].as_str().unwrap_or("Unknown");
        let index = device["index"].as_u64().expect("Device index is not a number").to_string();
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
    // original_host = cpal::host_from_id(cpal::HostId::Alsa).expect("Could not initialize audio routing using ALSA");
    // normal_output = original_host.default_output_device().expect("Could not get default output device");

    Command::new("pactl")
        .args(&["load-module", "module-null-sink", "sink_name=VirtualMic", "sink_properties=device.description=\"Virtual_Microphone\""])
        .output()
        .expect("Failed to execute process");
    Command::new("pactl")
        .args(&["load-module", "module-remap-source", "master=VirtualMic.monitor", "source_name=VirtualMicSource", "source_properties=device.description=\"Virtual_Mic_Source\""])
        .output()
        .expect("Failed to execute process");
    Command::new("pactl")
        .args(&["set-sink-volume", "VirtualMic", "100%"])
        .output()
        .expect("Failed to set sink volume");
    Command::new("pactl")
        .args(&["set-sink-volume", "VirtualMicSource", "100%"])
        .output()
        .expect("Failed to set sink volume");
    
    let host = cpal::host_from_id(cpal::HostId::Alsa).expect("Could not initialize audio routing using ALSA"); // Alsa needed so pulse default works
    let virtual_mic = host.default_output_device().expect("Could not get default output device");
    let virtual_mic_stream = OutputStreamBuilder::from_device(virtual_mic).expect("Unable to open default audio device").open_stream().expect("Failed to open stream");
    move_playback_to_sink();
    return virtual_mic_stream;
    // return (OutputStreamBuilder::from_device(normal_output).expect("Unable to open default audio device").open_stream().expect("Failed to open stream"), OutputStreamBuilder::from_device(virtual_mic).expect("Unable to open default audio device").open_stream().expect("Failed to open stream"));
}

pub fn reload_sound() {
    let script = r#"
        pactl list modules short | grep "Virtual_Microphone" | cut -f1 | xargs -L1 pactl unload-module
        pactl list modules short | grep "Virtual_Mic_Source" | cut -f1 | xargs -L1 pactl unload-module
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