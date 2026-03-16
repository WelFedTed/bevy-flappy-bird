use bevy::prelude::*;
use bevy::window::{WindowPlugin, WindowResolution};
use std::collections::HashMap;

const GRAVITY: f32 = -1000.0;
const JUMP_STRENGTH: f32 = 350.0;
const MAX_FALL_SPEED: f32 = -400.0;
const SCROLL_SPEED: f32 = 100.0;
const SPAWN_LOCATION: f32 = 336.0;
const DESPAWN_LOCATION: f32 = -336.0;
const PIPE_GAP: f32 = 200.0; // distance between pipes for bird to pass through
const PIPE_SPAWN_INTERVAL: f32 = 2.0;

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
        .add_systems(Startup, load_atlas)
        .add_systems(Startup, setup.after(load_atlas))
        .add_systems(Startup, spawn_player.after(load_atlas))
        .add_systems(Startup, spawn_ground.after(load_atlas))
        .add_systems(Startup, spawn_pipes.after(load_atlas))
        .add_systems(Update, animate_sprite)
        .add_systems(Update, apply_gravity)
        .add_systems(Update, player_movement)
        .add_systems(Update, player_jump)
        .add_systems(Update, player_rotation)
        .add_systems(Update, move_obstacles)
        .add_systems(Update, despawn_offscreen_entities)
        .run();
}

#[derive(Resource)]
struct Atlas {
    texture: Handle<Image>,
    layout: Handle<TextureAtlasLayout>,
    map: HashMap<String, usize>,
}

fn load_atlas(
    mut commands: Commands,
    asset_server: Res<AssetServer>,
    mut layouts: ResMut<Assets<TextureAtlasLayout>>,
) {
    let texture = asset_server.load("atlas.png");
    let txt = std::fs::read_to_string("assets/atlas.txt").unwrap();
    let texture_width = 1024; // width of flappy bird's atlas.png
    let texture_height = 1024; // height of flappy bird's atlas.png

    let mut layout = TextureAtlasLayout::new_empty(UVec2::new(texture_width, texture_height));
    let mut map = HashMap::new();

    for (i, line) in txt.lines().enumerate() {
        let parts: Vec<&str> = line.split_whitespace().collect();

        let name = parts[0];
        // let width: f32 = parts[1].parse().unwrap();
        // let height: f32 = parts[2].parse().unwrap();
        let u: f32 = parts[3].parse().unwrap();
        let v: f32 = parts[4].parse().unwrap();
        let uw: f32 = parts[5].parse().unwrap();
        let vh: f32 = parts[6].parse().unwrap();

        let x = (u * texture_width as f32) as u32;
        let y = (v * texture_height as f32) as u32;
        let w = (uw * texture_width as f32) as u32;
        let h = (vh * texture_height as f32) as u32;

        layout.add_texture(URect::new(x, y, x + w, y + h));

        map.insert(name.to_string(), i);
    }

    let layout_handle = layouts.add(layout);

    commands.insert_resource(Atlas {
        texture,
        layout: layout_handle,
        map,
    });
}

fn spawn_sprite(commands: &mut Commands, atlas: &Atlas, name: &str, pos: Vec3) {
    let index = atlas.map[name];

    commands.spawn((
        Sprite::from_atlas_image(
            atlas.texture.clone(),
            TextureAtlas {
                layout: atlas.layout.clone(),
                index,
            },
        ),
        Transform::from_translation(pos),
    ));
}

fn setup(mut commands: Commands, atlas: Res<Atlas>) {
    commands.spawn(Camera2d);

    spawn_sprite(&mut commands, &atlas, "bg_day", Vec3::new(0.0, 0.0, 0.0));
}

#[derive(Component)]
struct Animation {
    frames: Vec<usize>,
    timer: Timer,
    current: usize,
}

fn spawn_player(mut commands: Commands, atlas: Res<Atlas>) {
    let frames = vec![
        atlas.map["bird0_0"],
        atlas.map["bird0_1"],
        atlas.map["bird0_2"],
        atlas.map["bird0_1"],
    ];

    commands.spawn((
        Sprite::from_atlas_image(
            atlas.texture.clone(),
            TextureAtlas {
                layout: atlas.layout.clone(),
                index: frames[0],
            },
        ),
        Transform::from_xyz(-50.0, 0.0, 3.0),
        Animation {
            frames,
            timer: Timer::from_seconds(0.1, TimerMode::Repeating),
            current: 0,
        },
        Player,
        Velocity { y: 0.0 },
    ));
}

fn animate_sprite(time: Res<Time>, mut query: Query<(&mut Animation, &mut Sprite)>) {
    for (mut animation, mut sprite) in &mut query {
        animation.timer.tick(time.delta());

        if animation.timer.just_finished() {
            animation.current = (animation.current + 1) % animation.frames.len();

            if let Some(atlas) = &mut sprite.texture_atlas {
                atlas.index = animation.frames[animation.current];
            }
        }
    }
}

#[derive(Component)]
struct Player;

#[derive(Component)]
struct Velocity {
    y: f32,
}

fn player_movement(time: Res<Time>, mut query: Query<(&Velocity, &mut Transform), With<Player>>) {
    for (velocity, mut transform) in &mut query {
        transform.translation.y += velocity.y * time.delta_secs();
    }
}

fn apply_gravity(time: Res<Time>, mut query: Query<&mut Velocity, With<Player>>) {
    for mut velocity in &mut query {
        velocity.y += GRAVITY * time.delta_secs();
        velocity.y = velocity.y.max(MAX_FALL_SPEED);
    }
}

fn player_jump(keyboard: Res<ButtonInput<KeyCode>>, mut query: Query<&mut Velocity, With<Player>>) {
    if keyboard.just_pressed(KeyCode::Space) {
        for mut velocity in &mut query {
            velocity.y = JUMP_STRENGTH;
        }
    }
}

fn player_rotation(query: Query<(&Velocity, &mut Transform), With<Player>>) {
    for (velocity, mut transform) in query {
        let angle = (velocity.y / 600.0).clamp(-1.0, 1.0);
        transform.rotation = Quat::from_rotation_z(angle * 1.5);
    }
}

#[derive(Component)]
struct Ground;

fn spawn_ground(mut commands: Commands, atlas: Res<Atlas>) {
    commands.spawn((
        Sprite::from_atlas_image(
            atlas.texture.clone(),
            TextureAtlas {
                layout: atlas.layout.clone(),
                index: atlas.map["land"],
            },
        ),
        Transform::from_xyz(0.0, -200.0, 2.0),
        Ground,
    ));
}

fn move_obstacles(
    time: Res<Time>,
    mut query: Query<&mut Transform, Or<(With<Ground>, With<Pipe>)>>,
) {
    for mut transform in &mut query {
        transform.translation.x -= SCROLL_SPEED * time.delta_secs();
    }
}

fn despawn_offscreen_entities(mut commands: Commands, query: Query<(Entity, &Transform)>) {
    for (entity, transform) in &query {
        if transform.translation.x < DESPAWN_LOCATION {
            commands.entity(entity).despawn();
            // println!("REMOVED ENTITY");
        }
    }
}

#[derive(Component)]
struct Pipe;

fn spawn_pipes(mut commands: Commands, atlas: Res<Atlas>) {
    commands.spawn((
        Sprite::from_atlas_image(
            atlas.texture.clone(),
            TextureAtlas {
                layout: atlas.layout.clone(),
                index: atlas.map["pipe_up"],
            },
        ),
        Transform::from_xyz(SPAWN_LOCATION, 0.0, 1.0),
        Pipe,
    ));
}
