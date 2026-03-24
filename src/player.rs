use crate::Dead;
use crate::Player;
use crate::Velocity;
use crate::JUMP_STRENGTH;
use bevy::prelude::*;

pub struct PlayerPlugin;

impl Plugin for PlayerPlugin {
    fn build(&self, app: &mut App) {
        app.add_systems(Update, player_jump);
        app.add_systems(Update, player_rotation);
    }
}

fn player_rotation(query: Query<(&Velocity, &mut Transform), With<Player>>) {
    for (velocity, mut transform) in query {
        let angle = (velocity.y / 600.0).clamp(-1.0, 1.0);
        transform.rotation = Quat::from_rotation_z(angle * 1.5);
    }
}

fn player_jump(
    dead: Res<Dead>,
    keyboard: Res<ButtonInput<KeyCode>>,
    asset_server: Res<AssetServer>,
    mut commands: Commands,
    mut query: Query<&mut Velocity, With<Player>>,
) {
    if keyboard.just_pressed(KeyCode::Space) && !dead.0 {
        for mut velocity in &mut query {
            velocity.y = JUMP_STRENGTH;
        }
        let audio = asset_server.load("sfx_wing.ogg");
        commands.spawn((AudioPlayer::new(audio), PlaybackSettings::ONCE));
    }
}
