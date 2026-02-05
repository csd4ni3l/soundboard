use bevy::{log::Level, prelude::*};

use std::{collections::HashMap, fs::File, io::BufReader, path::Path};

use serde::{Deserialize, Serialize};

use bevy_egui::{EguiContextSettings, EguiContexts, EguiPrimaryContextPass, EguiStartupSet, egui};

use egui::ecolor::Color32;

#[cfg(target_os = "linux")]
mod linux_lib;

#[cfg(target_os = "windows")]
mod windows_lib;

use rodio::{
    Decoder, OutputStream, OutputStreamBuilder, Sink, Source,
    cpal::{self, traits::HostTrait},
};

#[derive(Serialize, Deserialize)]
struct JSONData {
    tabs: Vec<String>,
}

#[allow(dead_code)]
struct PlayingSound {
    file_path: String,
    length: f32,
    virtual_sink: Sink,
    // normal_sink: Sink
}

struct SoundSystem {
    virtual_mic_stream: OutputStream,
    // normal_output_stream: OutputStream,
    paused: bool,
}

#[derive(Resource)]
struct AppState {
    loaded_files: HashMap<String, Vec<String>>,
    json_data: JSONData,
    current_directory: String,
    currently_playing: Vec<PlayingSound>,
    sound_system: SoundSystem,
    virt_outputs: Vec<(String, String)>,
    virt_output_index_switch: String,
    virt_output_index: String,
}

const ALLOWED_FILE_EXTENSIONS: [&str; 4] = ["mp3", "wav", "flac", "ogg"];

fn create_virtual_mic() -> OutputStream {
    #[cfg(target_os = "windows")]
    return windows_lib::create_virtual_mic_windows();

    #[cfg(target_os = "linux")]
    return linux_lib::create_virtual_mic_linux();

    #[allow(unreachable_code)]
    {
        println!(
            "Unknown/unsupported OS. Audio support may not work or may route to default output (headset, headphones, etc)."
        );
        let host = cpal::default_host();
        let virtual_mic = host
            .default_output_device()
            .expect("Could not get default output device");
        return OutputStreamBuilder::from_device(virtual_mic)
            .expect("Unable to open default audio device")
            .open_stream()
            .expect("Failed to open stream");
        // normal_output = host.default_output_device().expect("Could not get default output device");
        // return (OutputStreamBuilder::from_device(normal_output).expect("Unable to open default audio device").open_stream().expect("Failed to open stream"), OutputStreamBuilder::from_device(virtual_mic).expect("Unable to open default audio device").open_stream().expect("Failed to open stream"));
    }
}

fn reload_sound() -> OutputStream {
    #[cfg(target_os = "linux")]
    linux_lib::reload_sound();

    return create_virtual_mic();
}

fn list_outputs() -> Vec<(String, String)> {
    #[cfg(target_os = "windows")]
    return Vec::from([("Select in apps".to_string(), String::from("9999999"))]);

    #[cfg(target_os = "linux")]
    return linux_lib::list_outputs();

    #[allow(unreachable_code)]
    return Vec::new();
}

fn main() {
    let virtual_mic_stream = create_virtual_mic();
    // let (normal_output_stream, virtual_mic_stream) = create_virtual_mic();

    App::new()
        .insert_resource(ClearColor(Color::BLACK))
        .add_plugins(
            DefaultPlugins
                .set(bevy::log::LogPlugin {
                    filter: "warn,ui=info".to_string(),
                    level: Level::INFO,
                    ..Default::default()
                })
                .set(WindowPlugin {
                    primary_window: Some(Window {
                        // You may want this set to `true` if you need virtual keyboard work in mobile browsers.
                        prevent_default_event_handling: false,
                        ..default()
                    }),
                    ..default()
                }),
        )
        .add_plugins(bevy_egui::EguiPlugin::default())
        .insert_resource(AppState {
            loaded_files: HashMap::new(),
            json_data: JSONData { tabs: Vec::new() },
            current_directory: String::new(),
            currently_playing: Vec::new(),
            sound_system: SoundSystem {
                virtual_mic_stream,
                // normal_output_stream,
                paused: false,
            },
            virt_outputs: Vec::new(),
            virt_output_index_switch: String::from("0"),
            virt_output_index: String::from("999"),
        })
        .add_systems(
            PreStartup,
            setup_camera_system.before(EguiStartupSet::InitContexts),
        )
        .add_systems(Startup, load_system)
        .add_systems(
            EguiPrimaryContextPass,
            (ui_system, update_ui_scale_factor_system, update_virtualmic),
        )
        .run();
}

fn update_virtualmic(mut app_state: ResMut<AppState>) {
    if app_state.virt_outputs.is_empty() {
        return;
    }

    if app_state.virt_output_index != app_state.virt_output_index_switch {
        app_state.virt_output_index = app_state.virt_output_index_switch.clone();
        #[cfg(target_os = "linux")]
        linux_lib::move_index_to_virtualmic(app_state.virt_output_index_switch.clone());
    }
}

fn load_system(mut app_state: ResMut<AppState>) {
    app_state.virt_outputs = list_outputs();
    if !app_state.virt_outputs.is_empty() {
        app_state.virt_output_index_switch = app_state.virt_outputs[0].1.clone();
    }
    load_data(&mut app_state);
}

fn load_data(app_state: &mut AppState) {
    if std::fs::exists("data.json").expect("Failed to check existence of JSON file") {
        let data = std::fs::read_to_string("data.json").expect("Failed to read JSON");
        app_state.json_data = serde_json::from_str(&data).expect("Failed to load JSON");

        let tabs = app_state.json_data.tabs.clone();
        app_state.loaded_files.clear();

        if tabs.len() > 0 {
            app_state.current_directory = tabs[0].clone();
        }

        for tab in tabs {
            app_state.loaded_files.insert(tab.clone(), Vec::new());
            if std::fs::exists(tab.clone()).expect("Failed to check existence of tab directory.") {
                app_state.loaded_files.insert(
                    tab.clone(),
                    std::fs::read_dir(tab)
                        .expect("Failed to read directory")
                        .filter_map(|entry| {
                            entry.ok().and_then(|e| {
                                let path = e.path();
                                if path.is_file()
                                    && ALLOWED_FILE_EXTENSIONS.contains(
                                        &path
                                            .extension()
                                            .unwrap_or_default()
                                            .to_str()
                                            .expect("Could not convert extension to string"),
                                    )
                                {
                                    path.to_str().map(|s| s.to_string())
                                } else {
                                    None
                                }
                            })
                        })
                        .collect(),
                );
            }
        }
    }
}

fn setup_camera_system(mut commands: Commands) {
    commands.spawn(Camera2d);
}

fn update_ui_scale_factor_system(egui_context: Single<(&mut EguiContextSettings, &Camera)>) {
    let (mut egui_settings, camera) = egui_context.into_inner();
    egui_settings.scale_factor = 1.5 / camera.target_scaling_factor().unwrap_or(1.5);
}

fn play_sound(file_path: String, app_state: &mut AppState) {
    let virtual_file = File::open(&file_path).unwrap();
    let virtual_src = Decoder::new(BufReader::new(virtual_file)).unwrap();
    let virtual_sink = Sink::connect_new(&app_state.sound_system.virtual_mic_stream.mixer());
    let length = virtual_src
        .total_duration()
        .expect("Could not get source duration")
        .as_secs_f32();
    virtual_sink.append(virtual_src);
    virtual_sink.play();

    // let normal_file = File::open(&file_path).unwrap();
    // let normal_src = Decoder::new(BufReader::new(normal_file)).unwrap();
    // let normal_sink = Sink::connect_new(&app_state.sound_system.normal_output_stream.mixer());
    // normal_sink.append(normal_src);
    // normal_sink.play();

    app_state.currently_playing.push(PlayingSound {
        file_path: file_path.clone(),
        length,
        virtual_sink,
        // normal_sink
    })
}

fn ui_system(mut contexts: EguiContexts, mut app_state: ResMut<AppState>) -> Result {
    let ctx = contexts.ctx_mut()?;

    egui::TopBottomPanel::top("top_panel").show(ctx, |ui| {
        ui.heading("csd4ni3l Soundboard");
    });

    egui::SidePanel::right("tools").show(ctx, |ui| {
        ui.heading("Tools");

        ui.separator();

        let available_width = ui.available_width();
        let available_height = ui.available_height();
        let outputs = app_state.virt_outputs.clone();

        let mut mic_name = "Select inside apps".to_string();

        #[cfg(target_os = "linux")]
        {
            let output_index = app_state.virt_output_index.clone();
            let output_device = linux_lib::get_device_by_index("source-outputs", output_index);
            if let Some(app_name) = output_device["properties"]["application.name"].as_str() {
                mic_name = app_name.to_string();
            }
        }

        ui.label("Virtual Mic Output");

        egui::ComboBox::from_id_salt("Virtual Mic Output")
            .selected_text(mic_name)
            .width(available_width)
            .height(available_height / 15.0)
            .show_ui(ui, |ui| {
                for output in &outputs {
                    ui.selectable_value(
                        &mut app_state.virt_output_index_switch,
                        output.1.clone(),
                        output.0.clone(),
                    );
                }
            });

        if ui
            .add_sized(
                [available_width, available_height / 15.0],
                egui::Button::new("Add folder"),
            )
            .clicked()
        {
            if let Some(folder) = rfd::FileDialog::new().pick_folder() {
                if let Some(path_str) = folder.to_str() {
                    println!("Selected: {}", path_str);
                    app_state.json_data.tabs.push(path_str.to_string());
                    std::fs::write(
                        "data.json",
                        serde_json::to_string(&app_state.json_data)
                            .expect("Could not convert JSON to string"),
                    )
                    .expect("Could not write to JSON file");
                    load_data(&mut app_state);
                } else {
                    println!("Invalid path encoding!");
                }
            }
        }

        if ui
            .add_sized(
                [available_width, available_height / 15.0],
                egui::Button::new("Reload content"),
            )
            .clicked()
        {
            load_data(&mut app_state);
            println!("Reloaded content");
        }

        if ui
            .add_sized(
                [available_width, available_height / 15.0],
                egui::Button::new("Youtube downloader"),
            )
            .clicked()
        {
            println!("Youtube downloader!");
        }

        if ui
            .add_sized(
                [available_width, available_height / 15.0],
                egui::Button::new("Reload sound system"),
            )
            .clicked()
        {
            app_state.currently_playing.clear();
            app_state.sound_system.virtual_mic_stream = reload_sound();
            // (app_state.sound_system.normal_output_stream, app_state.sound_system.virtual_mic_stream) = reload_sound();
            println!("Sucessfully reloaded sound system!");
        }
    });

    egui::TopBottomPanel::bottom("currently_playing").show(ctx, |ui| {
        ui.horizontal(|ui| {
            if app_state.sound_system.paused {
                ui.heading("Paused");
            } else {
                ui.heading("Playing");
            }

            ui.vertical(|ui| {
                for playing_sound in &app_state.currently_playing {
                    ui.label(format!(
                        "{} - {:.2} / {:.2}",
                        playing_sound.file_path,
                        playing_sound.virtual_sink.get_pos().as_secs_f32(),
                        playing_sound.length
                    ));
                }
            })
        });
    });

    egui::CentralPanel::default().show(ctx, |ui| {
        let available_height = ui.available_height();
        ui.horizontal(|ui| {
            let available_width = ui.available_width();
            let current_directories = app_state.loaded_files.keys().cloned().collect::<Vec<_>>();
            for directory in current_directories.clone() {
                let mut button = egui::Button::new(&directory);
                if directory == app_state.current_directory {
                    button = button.fill(Color32::BLACK);
                }

                if ui
                    .add_sized(
                        [
                            available_width / current_directories.len() as f32,
                            available_height / 15.0,
                        ],
                        button,
                    )
                    .clicked()
                {
                    app_state.current_directory = directory;
                };
            }
        });
        ui.add_space(available_height / 50.0);
        if app_state.current_directory.chars().count() > 0 {
            let files = app_state
                .loaded_files
                .get(&app_state.current_directory)
                .cloned()
                .unwrap_or_default();

            egui::ScrollArea::vertical().show(ui, |ui| {
                for element in files {
                    if let Some(filename) = element.split("/").collect::<Vec<_>>().last() {
                        if ui
                            .add_sized(
                                [ui.available_width(), available_height / 15.0],
                                egui::Button::new(*filename),
                            )
                            .clicked()
                        {
                            let path = Path::new(&app_state.current_directory)
                                .join(filename)
                                .to_string_lossy()
                                .to_string();
                            play_sound(path, &mut app_state);
                        }
                    }
                }
            });
        }
    });

    app_state.currently_playing.retain(|playing_sound| {
        playing_sound.virtual_sink.get_pos().as_secs_f32() <= (playing_sound.length - 0.01) // 0.01 offset needed here because of floating point errors and so its not exact
    });

    Ok(())
}
