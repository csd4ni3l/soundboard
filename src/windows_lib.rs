use rodio::{
    OutputStream, OutputStreamBuilder,
    cpal::{self, traits::{DeviceTrait, StreamTrait, HostTrait}, StreamConfig, SampleRate},
};

use ringbuf::{traits::*, HeapRb};

fn route_standard_to_virtual(host: &cpal::Host, virtual_mic: &cpal::Device) {
    let standard_mic = host.default_input_device().expect("Could not get default input device.");

    let config = StreamConfig {
        channels: 2,
        sample_rate: SampleRate(48_000),
        buffer_size: cpal::BufferSize::Default,
    };
    let rb = HeapRb::<f32>::new(48_000 * 2);
    let (mut producer, mut consumer) = rb.split();

    let input_stream = standard_mic.build_input_stream(
        &config,
        move |data: &[f32], _| {
            for &sample in data {
                let _ = producer.try_push(sample);
                let _ = producer.try_push(sample);
            }
        },
        move |err| eprintln!("Input stream error: {err}"),
        None,
    ).expect("Could not build input stream for standard to virtual mic routing");

    let output_stream = virtual_mic.build_output_stream(
        &config,
        move |data: &mut [f32], _| {
            for sample in data {
                *sample = consumer.try_pop().unwrap_or(0.0);
            }
        },
        move |err| eprintln!("Output stream error: {err}"),
        None,
    ).expect("Could not build output stream for standard to virtual mic routing");

    input_stream.play();
    output_stream.play();
}

pub fn create_virtual_mic_windows() -> (OutputStream, OutputStream) {
    let host = cpal::host_from_id(cpal::HostId::Wasapi)
        .expect("Could not initialize audio routing using WasAPI");

    let virtual_mic = host
        .output_devices()
        .expect("Could not list Output devices")
        .find(|device| {
            device
                .name()
                .ok()
                .map(|name| name.contains("CABLE Input") || name.contains("VB-Audio"))
                .unwrap_or(false)
        })
        .expect("Could not get VB Cable output device. Is VB Cable Driver installed?");

    route_standard_to_virtual(&host, &virtual_mic);

    let normal_output = host
        .default_output_device()
        .expect("Could not get default output device");

    return (
        OutputStreamBuilder::from_device(normal_output)
            .expect("Unable to open default audio device")
            .open_stream()
            .expect("Failed to open stream"),
        OutputStreamBuilder::from_device(virtual_mic)
            .expect("Unable to open default audio device")
            .open_stream()
            .expect("Failed to open stream"),
    );
}