use bevy::{log::Level, prelude::*};

use std::{collections::HashMap, fs::File, io::BufReader, path::Path, time::Instant};

use serde::{Deserialize, Serialize};

use bevy_egui::{EguiContextSettings, EguiContexts, EguiPrimaryContextPass, EguiStartupSet, egui::{self, Context, Ui}};

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
    sink: Sink,
    to_remove: bool,
    #[cfg(target_os = "windows")]
    normal_sink: Sink,
}

struct SoundSystem {
    #[cfg(target_os = "windows")]
    normal_output_stream: OutputStream,
    output_stream: OutputStream,
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
    last_virt_output_update: Instant,
    current_view: String
}

const ALLOWED_FILE_EXTENSIONS: [&str; 4] = ["mp3", "wav", "flac", "ogg"];

fn create_virtual_mic() -> SoundSystem {
    #[cfg(target_os = "windows")]
    {
        let (normal, virtual_mic) = windows_lib::create_virtual_mic_windows();
        return SoundSystem {
            output_stream: virtual_mic,
            normal_output_stream: normal,
        };
    }

    #[cfg(target_os = "linux")]
    {
        return SoundSystem {
            output_stream: linux_lib::create_virtual_mic_linux(),
        };
    }

    #[allow(unreachable_code)]
    {
        let host = cpal::default_host();
        let device = host
            .default_output_device()
            .expect("Could not get default output device");
        SoundSystem {
            output_stream: OutputStreamBuilder::from_device(device)
                .expect("Unable to open device")
                .open_stream()
                .expect("Failed to open stream"),
            // this is actually not needed here, since windows would exit by far. But, cargo doesnt like SoundSystem not getting the normal_output stream so...
            #[cfg(target_os = "windows")]
            normal_output_stream: OutputStreamBuilder::from_device(device)
                .expect("Unable to open device")
                .open_stream()
                .expect("Failed to open stream"),
        }
    }
}

fn reload_sound() -> SoundSystem {
    #[cfg(target_os = "linux")]
    linux_lib::reload_sound();

    return create_virtual_mic();
}

fn list_outputs() -> Vec<(String, String)> {
    #[cfg(target_os = "windows")]
    return Vec::from([("Select inside apps".to_string(), String::from("9999999"))]);

    #[cfg(target_os = "linux")]
    return linux_lib::list_outputs();

    #[allow(unreachable_code)]
    return Vec::new();
}

fn main() {
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
            sound_system: create_virtual_mic(),
            virt_outputs: Vec::new(),
            virt_output_index_switch: String::from("0"),
            virt_output_index: String::from("999"),
            current_view: "main".to_string(),
            last_virt_output_update: Instant::now()
        })
        .add_systems(
            PreStartup,
            setup_camera_system.before(EguiStartupSet::InitContexts),
        )
        .add_systems(Startup, load_system)
        .add_systems(
            EguiPrimaryContextPass,
            (draw, update_ui_scale_factor_system, update),
        )
        .run();
}

fn update(mut app_state: ResMut<AppState>) {
    if app_state.last_virt_output_update.elapsed().as_secs_f32() >= 3.0 {
        app_state.last_virt_output_update = Instant::now();
        app_state.virt_outputs = list_outputs();    
    }

    if app_state.virt_outputs.is_empty() {
        return;
    }

    if !(app_state.virt_output_index == "999".to_string()) {
        app_state.virt_output_index_switch = app_state.virt_outputs[0].1.clone();
    }

    if app_state.virt_output_index != app_state.virt_output_index_switch {
        app_state.virt_output_index = app_state.virt_output_index_switch.clone();
        #[cfg(target_os = "linux")]
        linux_lib::move_index_to_virtualmic(app_state.virt_output_index_switch.clone());
    }
}

fn load_system(mut app_state: ResMut<AppState>) {   
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
    let file = File::open(&file_path).unwrap();
    let src = Decoder::new(BufReader::new(file)).unwrap();
    let length = src
        .total_duration()
        .expect("Could not get source duration")
        .as_secs_f32();

    let sink = Sink::connect_new(&app_state.sound_system.output_stream.mixer());
    sink.append(src);
    sink.play();

    let playing_sound = PlayingSound {
        file_path: file_path.clone(),
        length,
        sink,
        to_remove: false,
        #[cfg(target_os = "windows")]
        normal_sink: {
            let file2 = File::open(&file_path).unwrap();
            let src2 = Decoder::new(BufReader::new(file2)).unwrap();
            let normal_sink =
                Sink::connect_new(&app_state.sound_system.normal_output_stream.mixer());
            normal_sink.append(src2);
            normal_sink.play();
            normal_sink
        },
    };

    app_state.currently_playing.push(playing_sound);
}

fn create_virtual_mic_dropdown(ui: &mut Ui, app_state: &mut ResMut<AppState>, available_width: f32, available_height: f32) {
    #[cfg(target_os = "linux")] {
        let outputs = app_state.virt_outputs.clone();
        let output_index = app_state.virt_output_index.clone();
        let output_sink = linux_lib::get_sink_by_index("source-outputs", output_index);
        if let Some(app_name) = output_sink["properties"]["application.name"].as_str() {
            egui::ComboBox::from_id_salt("Virtual Mic Output")
                .selected_text(app_name.to_string())
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
        }
        else {
            ui.add(egui::Button::new("No apps found to use.".to_string()));
        }

        return;
    }
    #[allow(unreachable_code)]
    {
        ui.add(egui::Button::new("Unsupported. Select inside apps.".to_string()));
    }
}

fn main_ui(ctx: &Context, mut app_state: ResMut<AppState>) {
    egui::SidePanel::right("tools").show(ctx, |ui| {
        ui.heading("Tools");

        ui.separator();

        let available_width = ui.available_width();
        let available_height = ui.available_height();
        ui.label("Virtual Mic Output");
        create_virtual_mic_dropdown(ui, &mut app_state, available_width, available_height);

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
            app_state.current_view = "youtube_downloader".to_string();
        }

        if ui
            .add_sized(
                [available_width, available_height / 15.0],
                egui::Button::new("Reload sound system"),
            )
            .clicked()
        {
            app_state.currently_playing.clear();
            app_state.sound_system = reload_sound();
            println!("Sucessfully reloaded sound system!");
        }
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
}

fn youtube_downloader_ui(ctx: &Context, app_state: ResMut<AppState>) {
    egui::CentralPanel::default().show(ctx, |ui| {
        ui.heading(format!("Coming Soon! Currently on {} view.", app_state.current_view)); // view is only included here so there is no warning about app_state not being used.
    });
}

fn draw(mut contexts: EguiContexts, mut app_state: ResMut<AppState>) -> Result {
    let ctx = contexts.ctx_mut()?;

    egui::TopBottomPanel::top("top_panel").show(ctx, |ui| {
        ui.heading("csd4ni3l Soundboard");
    });

    egui::TopBottomPanel::bottom("currently_playing").show(ctx, |ui| {
        ui.vertical(|ui| {
            for playing_sound in &mut app_state.currently_playing {
                ui.horizontal(|ui| {
                    ui.label(format!(
                        "{} - {:.2} / {:.2}",
                        playing_sound.file_path,
                        playing_sound.sink.get_pos().as_secs_f32(),
                        playing_sound.length
                    ));
                    let available_width = ui.available_width();
                    let available_height = ui.available_height();
                    if ui
                        .add_sized(
                            [
                                available_width / 2 as f32,
                                available_height,
                            ],
                            egui::Button::new("Stop"),
                        )
                        .clicked()
                    {
                        playing_sound.to_remove = true;
                    };
                    if ui
                        .add_sized(
                            [
                                available_width / 2 as f32,
                                available_height,
                            ],
                            egui::Button::new(if playing_sound.sink.is_paused() {"Resume"} else {"Pause"}),
                        )
                        .clicked()
                    {
                        if playing_sound.sink.is_paused() {
                            playing_sound.sink.play();
                        }
                        else {
                            playing_sound.sink.pause();
                        }
                    };
                });
            }
        });
        
        let available_width = ui.available_width();
        let available_height = ui.available_height();

        if ui
            .add_sized(
                [available_width, available_height / 15.0],
                egui::Button::new("Stop all"),
            )
            .clicked()
        {
            app_state.currently_playing.clear();
        }
    });
    
    app_state.currently_playing.retain(|playing_sound| { // retains happen the next cycle, not in the current one because of borrowing and im lazy to fix
        playing_sound.sink.get_pos().as_secs_f32() <= (playing_sound.length - 0.01) && !playing_sound.to_remove  // 0.01 offset needed here because of floating point errors and so its not exact
    });
    
    if app_state.current_view == "main".to_string() {
        main_ui(ctx, app_state);
    }
    else if app_state.current_view == "youtube_downloader".to_string() {
        youtube_downloader_ui(ctx, app_state);
    }

    Ok(())
}
