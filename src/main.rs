use bevy::{
    log::{Level, LogPlugin},
    prelude::*,
};

use std::collections::HashMap;

use serde::{Deserialize, Serialize};

use bevy_egui::{
    EguiContextSettings, EguiContexts, EguiPlugin, EguiPrimaryContextPass, EguiStartupSet, egui,
};
#[derive(Serialize, Deserialize)]
struct JSONData {
    tabs: Vec<String>,
}

#[derive(Resource)]
struct FileData {
    loaded_files: HashMap<String, Vec<String>>,
    json_data: JSONData,
}

fn main() {
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
        .insert_resource(FileData {
            loaded_files: HashMap::new(),
            json_data: JSONData { tabs: Vec::new() },
        })
        .add_systems(
            PreStartup,
            setup_camera_system.before(EguiStartupSet::InitContexts),
        )
        .add_systems(Startup, load_json_system)
        .add_systems(
            EguiPrimaryContextPass,
            (ui_system, update_ui_scale_factor_system),
        )
        .run();
}

fn load_json_system(mut file_data: ResMut<FileData>) {
    if std::fs::exists("data.json").expect("Failed to check existence of JSON file") {
        let data = std::fs::read_to_string("data.json").expect("Failed to read JSON");
        file_data.json_data = serde_json::from_str(&data).expect("Failed to load JSON");

        let tabs = file_data.json_data.tabs.clone();
        file_data.loaded_files.clear();

        for tab in tabs {
            file_data.loaded_files.insert(tab.clone(), Vec::new());
            if std::fs::exists(tab.clone()).expect("Failed to check existence of tab directory.") {
                file_data.loaded_files.insert(
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
    keyboard_input: Res<ButtonInput<KeyCode>>,
    mut toggle_scale_factor: Local<Option<bool>>,
    egui_context: Single<(&mut EguiContextSettings, &Camera)>,
) {
    let (mut egui_settings, camera) = egui_context.into_inner();
    if keyboard_input.just_pressed(KeyCode::Slash) || toggle_scale_factor.is_none() {
        *toggle_scale_factor = Some(!toggle_scale_factor.unwrap_or(true));

        let scale_factor = if toggle_scale_factor.unwrap() {
            1.0
        } else {
            1.0 / camera.target_scaling_factor().unwrap_or(1.0)
        };
        egui_settings.scale_factor = scale_factor;
    }
}

fn ui_system(mut contexts: EguiContexts, mut file_data: ResMut<FileData>) -> Result {
    let ctx = contexts.ctx_mut()?;

    egui::TopBottomPanel::top("top_panel").show(ctx, |ui| {
        ui.heading("csd4ni3l Soundboard");
    });

    egui::CentralPanel::default().show(ctx, |ui| {
        ui.label("The app!");
    });

    egui::SidePanel::right("tools").show(ctx, |ui| {
        ui.heading("Tools");

        ui.separator();

        if ui
            .add_sized(
                [ui.available_width(), 40.0],
                egui::Button::new("Add folder"),
            )
            .clicked()
        {
            if let Some(folder) = rfd::FileDialog::new().pick_folder() {
                if let Some(path_str) = folder.to_str() {
                    println!("Selected: {}", path_str);
                    file_data.json_data.tabs.push(path_str.to_string());
                    std::fs::write(
                        "data.json",
                        serde_json::to_string(&file_data.json_data)
                            .expect("Could not convert JSON to string"),
                    )
                    .expect("Could not write to JSON file");
                    load_json_system(file_data);
                } else {
                    println!("Invalid path encoding!");
                }
            }
        }

        if ui
            .add_sized(
                [ui.available_width(), 40.0],
                egui::Button::new("Reload content"),
            )
            .clicked()
        {
            load_json_system(file_data);
            println!("Reloaded content");
        }

        if ui
            .add_sized(
                [ui.available_width(), 40.0],
                egui::Button::new("Youtube downloader"),
            )
            .clicked()
        {
            println!("Youtube downloader!");
        }
    });

    Ok(())
}
