use bevy::prelude::*;
use bevy::window::{WindowPlugin, WindowResolution};

fn main() {
    App::new()
        .add_plugins(DefaultPlugins.set(WindowPlugin {
            primary_window: Some(Window {
                title: "bevy-flappy-bird".into(),
                resolution: WindowResolution::new(288, 512),
                ..default()
            }),
            ..default()
        }))
        .add_systems(Update, hello_world)
        .run();
}

fn hello_world() {
    println!("hello world!");
}
