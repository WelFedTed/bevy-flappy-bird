use bevy::prelude::*;
use bevy::window::{WindowPlugin, WindowResolution};

fn main() {
    App::new()
        .add_plugins(
            DefaultPlugins
                .set(WindowPlugin {
                    primary_window: Some(Window {
                        title: "bevy-flappy-bird".into(),
                        resolution: WindowResolution::new(288, 512),
                        ..default()
                    }),
                    ..default()
                })
                .set(ImagePlugin::default_nearest()),
        )
        .add_systems(Startup, setup)
        // .add_systems(Update, hello_world)
        .run();
}

fn setup(mut commands: Commands, asset_server: Res<AssetServer>) {
    commands.spawn(Camera2d);
    commands.spawn((
        Sprite::from_image(asset_server.load("gfx/atlas.png")),
        Transform::from_xyz(370.0, -255.0, 0.0), //not sure why these values work, need to change to texture atlast
    ));
}
