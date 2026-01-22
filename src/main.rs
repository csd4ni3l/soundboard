use bevy::{
    log::{Level, LogPlugin},
    prelude::*,
};

use std::{collections::HashMap, fs::File, io::BufReader, path::Path};

use serde::{Deserialize, Serialize};

use bevy_egui::{
    EguiContextSettings, EguiContexts, EguiPlugin, EguiPrimaryContextPass, EguiStartupSet, egui,
};

use rodio::{Decoder, OutputStream, OutputStreamBuilder, Sink, mixer::Mixer};

#[derive(Serialize, Deserialize)]
struct JSONData {
    tabs: Vec<String>,
}

struct PlayingSound {
    file_path: String,
    start_time: f32,
}

struct SoundSystem {
    sink: Sink,
    paused: bool
}

#[derive(Resource)]
struct AppState {
    loaded_files: HashMap<String, Vec<String>>,
    json_data: JSONData,
    current_directory: String,
    currently_playing: Vec<PlayingSound>,
    sound_system: SoundSystem
}

use std::time::{SystemTime, UNIX_EPOCH};

fn main() {
    let stream_handle = OutputStreamBuilder::open_default_stream().expect("Unable to open default audio device");
    let mixer = stream_handle.mixer();
    let sink = Sink::connect_new(&mixer);

    App::new()
        .insert_resource(ClearColor(Color::BLACK))
        .add_plugins(
            DefaultPlugins
                .set(LogPlugin {
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
        .add_plugins(EguiPlugin::default())
        .insert_resource(AppState {
            loaded_files: HashMap::new(),
            json_data: JSONData { tabs: Vec::new() },
            current_directory: String::new(),
            currently_playing: Vec::new(),
            sound_system: SoundSystem {
                sink,
                paused: false
            }
        })
        .add_systems(
            PreStartup,
            setup_camera_system.before(EguiStartupSet::InitContexts),
        )
        .add_systems(Startup, load_system)
        .add_systems(
            EguiPrimaryContextPass,
            (ui_system, update_ui_scale_factor_system),
        )
        .run();
}

fn load_system(mut app_state: ResMut<AppState>) {
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
                                if path.is_file() {
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

fn update_ui_scale_factor_system(
    egui_context: Single<(&mut EguiContextSettings, &Camera)>,
) {
    let (mut egui_settings, camera) = egui_context.into_inner();
    egui_settings.scale_factor = 1.5 / camera.target_scaling_factor().unwrap_or(1.5);
}

fn play_sound(file_path: String, app_state: &mut AppState) {
    let file = BufReader::new(File::open(&file_path).unwrap());
    let src = Decoder::new(file).unwrap();
    app_state.sound_system.sink.append(src);

    let start = SystemTime::now();
    let since_the_epoch = start
        .duration_since(UNIX_EPOCH)
        .expect("time should go forward");
    
    app_state.currently_playing.push(PlayingSound { 
        file_path: file_path.clone(), 
        start_time: since_the_epoch.as_secs_f32(), 
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

        let available_height = ui.available_height();

        if ui
            .add_sized(
                [ui.available_width(), available_height / 15.0],
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
                [ui.available_width(), available_height / 15.0],
                egui::Button::new("Reload content"),
            )
            .clicked()
        {
            load_data(&mut app_state);
            println!("Reloaded content");
        }

        if ui
            .add_sized(
                [ui.available_width(), available_height / 15.0],
                egui::Button::new("Youtube downloader"),
            )
            .clicked()
        {
            println!("Youtube downloader!");
        }
    });

    egui::CentralPanel::default().show(ctx, |ui| {
        let available_height = ui.available_height();

        ui.horizontal(|ui| {
            let available_width = ui.available_width();
            let current_directories = app_state.loaded_files.keys().cloned().collect::<Vec<_>>();
            for directory in current_directories.clone() {
                if ui
                    .add_sized(
                        [available_width / current_directories.len() as f32, available_height / 15.0],
                        egui::Button::new(&directory),
                    )
                    .clicked()
                {
                    app_state.current_directory = directory;
                };
            }
        });
        ui.add_space(available_height / 50.0);
        ui.with_layout(egui::Layout::top_down(egui::Align::Center), |ui| {
            ui.label(egui::RichText::new(format!("The current directory is {}", app_state.current_directory)).font(egui::FontId::proportional(20.0)));
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
                        if ui.add_sized(
                            [ui.available_width(), available_height / 15.0],
                            egui::Button::new(*filename),
                        ).clicked() {
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

    Ok(())
}