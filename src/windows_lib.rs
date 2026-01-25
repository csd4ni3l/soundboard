use rodio::{OutputStream, OutputStreamBuilder, cpal::{self, traits::HostTrait}};

pub fn create_virtual_mic_windows() -> OutputStream {
    let host = cpal::host_from_id(cpal::HostId::Wasapi).expect("Could not initialize audio routing using WasAPI");
    let virtual_mic = host.output_devices().expect("Could not list Output devices").find(|device| {
        device.name().ok().map(|name|{
            name.contains("CABLE Input") || name.contains("VB-Audio")
        }).unwrap_or(false)
    }).expect("Could not get default output device");
    // normal_output = host.default_output_device().expect("Could not get default output device");
    return OutputStreamBuilder::from_device(virtual_mic).expect("Unable to open default audio device").open_stream().expect("Failed to open stream");
    // return (OutputStreamBuilder::from_device(normal_output).expect("Unable to open default audio device").open_stream().expect("Failed to open stream"), OutputStreamBuilder::from_device(virtual_mic).expect("Unable to open default audio device").open_stream().expect("Failed to open stream"));
}