use bevy::prelude::*;

fn spawn_camera(mut commands: Commands) {
    commands.spawn(Camera2d::default());
}

pub fn setup(mut commands: Commands) {
    spawn_camera(commands);
}

pub fn update(mut commands: Commands) {

}

fn main() {
    App::new()
        .add_plugins(DefaultPlugins)
        .add_systems(Startup, setup)
        .add_systems(Update, update)
        .run();
}

#[derive(Component)]
pub struct Person {
    pub name: String
}