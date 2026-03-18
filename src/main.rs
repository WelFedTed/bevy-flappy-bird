use bevy::color::palettes::css::*;
use bevy::math::bounding::*;
use bevy::prelude::*;
use bevy::sprite::Anchor;
use bevy::window::{WindowPlugin, WindowResolution};
use std::collections::HashMap;

const SCREEN_WIDTH: f32 = 288.0;
const SCREEN_HEIGHT: f32 = 512.0;
const GRAVITY: f32 = -1000.0;
const JUMP_STRENGTH: f32 = 350.0;
const MAX_FALL_SPEED: f32 = -400.0;
const SCROLL_SPEED: f32 = 100.0;
const PIPE_GAP: f32 = 50.0; // distance between pipes vertically for bird to pass through
const PIPE_INTERVAL: f32 = 175.0; // distance between pipe pairs horizontally
const GROUND_WIDTH: f32 = 288.0;
const GROUND_HEIGHT: f32 = 112.0;
const PIPE_WIDTH: f32 = 52.0;
const PIPE_HEIGHT: f32 = 320.0;
const PLAYER_RADIUS: f32 = 15.0; // radius of the player's collision circle
const DRAW_DEBUG: bool = false; // toggle to draw debug gizmos for collision detection

fn main() {
    App::new()
        .add_plugins(
            DefaultPlugins
                .set(WindowPlugin {
                    primary_window: Some(Window {
                        title: "bevy-flappy-bird".into(),
                        resolution: WindowResolution::new(
                            SCREEN_WIDTH as u32,
                            SCREEN_HEIGHT as u32,
                        ),
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
        .add_systems(Update, check_for_collisions)
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
        Anchor::CENTER,
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
        Anchor::CENTER,
        Animation {
            frames,
            timer: Timer::from_seconds(0.1, TimerMode::Repeating),
            current: 0,
        },
        Player,
        Velocity { y: 0.0 },
        CurrentVolume::Circle(BoundingCircle::new(Vec2::new(0.0, 0.0), 48.0 / 2.0)), // 48 px diameter
        Intersects(false),
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

fn player_jump(
    keyboard: Res<ButtonInput<KeyCode>>,
    asset_server: Res<AssetServer>,
    mut commands: Commands,
    mut query: Query<&mut Velocity, With<Player>>,
) {
    if keyboard.just_pressed(KeyCode::Space) {
        for mut velocity in &mut query {
            velocity.y = JUMP_STRENGTH;
        }
        let audio = asset_server.load("sfx_wing.ogg");
        commands.spawn((AudioPlayer::new(audio), PlaybackSettings::ONCE));
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
        Transform::from_xyz(0.0, -512.0 / 2.0, 2.0),
        Anchor::BOTTOM_CENTER,
        Ground,
        Obstacle,
    ));
    spawn_next_ground(&mut commands, &atlas);
}

fn move_obstacles(
    time: Res<Time>,
    mut query: Query<&mut Transform, Or<(With<Ground>, With<Pipe>)>>,
) {
    for mut transform in &mut query {
        transform.translation.x -= SCROLL_SPEED * time.delta_secs();
    }
}

fn spawn_next_ground(commands: &mut Commands, atlas: &Res<Atlas>) {
    // println!("SPAWNED NEXT GROUND");
    commands.spawn((
        Sprite::from_atlas_image(
            atlas.texture.clone(),
            TextureAtlas {
                layout: atlas.layout.clone(),
                index: atlas.map["land"],
            },
        ),
        Transform::from_xyz(SCREEN_WIDTH / 2.0 + 336.0 / 2.0, -512.0 / 2.0, 2.0),
        Ground,
        Obstacle,
        CurrentVolume::Aabb(Aabb2d::new(
            Vec2::new(0.0, GROUND_HEIGHT),
            Vec2::new(GROUND_WIDTH / 2.0, GROUND_HEIGHT / 2.0),
        )),
        Intersects(false),
        Anchor::BOTTOM_CENTER,
    ));
}

fn despawn_offscreen_entities(
    mut commands: Commands,
    atlas: Res<Atlas>,
    query: Query<(
        Entity,
        &Transform,
        Option<&Ground>,
        Option<&Pipe>,
        Option<&PipeTop>,
    )>,
) {
    for (entity, transform, maybe_ground, maybe_pipe, maybe_pipe_top) in &query {
        if let Some(_) = maybe_ground {
            if transform.translation.x <= (-SCREEN_WIDTH - 336.0) / 2.0 {
                // note: ground sprite width = 336.0
                commands.entity(entity).despawn();
                // println!("REMOVED GROUND");
                spawn_next_ground(&mut commands, &atlas);
            }
        }
        if let Some(_) = maybe_pipe {
            if transform.translation.x <= -(SCREEN_WIDTH + 52.0) / 2.0 {
                // note: pipe sprite width = 52.0
                commands.entity(entity).despawn();
                // println!("REMOVED PIPE");
                if let Some(_) = maybe_pipe_top {
                    spawn_next_pipes(&mut commands, &atlas);
                }
            }
        }
    }
}

#[derive(Component)]
struct Pipe;

#[derive(Component)]
struct PipeTop;

#[derive(Component)]
struct PipeBottom;

fn spawn_pipes(mut commands: Commands, atlas: Res<Atlas>) {
    for i in 0..2 {
        let pipe_offset: f32 = rand::random_range(-75.0..185.0);
        commands.spawn((
            Sprite::from_atlas_image(
                atlas.texture.clone(),
                TextureAtlas {
                    layout: atlas.layout.clone(),
                    index: atlas.map["pipe_up"],
                },
            ),
            Transform::from_xyz(
                (SCREEN_WIDTH + 52.0) / 2.0 + PIPE_INTERVAL * i as f32,
                -PIPE_GAP + pipe_offset,
                1.0,
            ),
            Pipe,
            PipeBottom,
            Obstacle,
            Anchor::TOP_CENTER,
        ));
        commands.spawn((
            Sprite::from_atlas_image(
                atlas.texture.clone(),
                TextureAtlas {
                    layout: atlas.layout.clone(),
                    index: atlas.map["pipe_down"],
                },
            ),
            Transform::from_xyz(
                (SCREEN_WIDTH + 52.0) / 2.0 + PIPE_INTERVAL * i as f32,
                PIPE_GAP + pipe_offset,
                1.0,
            ),
            Pipe,
            PipeTop,
            Obstacle,
            Anchor::BOTTOM_CENTER,
        ));
    }
}

fn spawn_next_pipes(commands: &mut Commands, atlas: &Res<Atlas>) {
    // println!("SPAWNED NEXT PIPES");
    let pipe_offset: f32 = rand::random_range(-75.0..185.0);
    commands.spawn((
        Sprite::from_atlas_image(
            atlas.texture.clone(),
            TextureAtlas {
                layout: atlas.layout.clone(),
                index: atlas.map["pipe_up"],
            },
        ),
        Transform::from_xyz((SCREEN_WIDTH + 52.0) / 2.0, -PIPE_GAP + pipe_offset, 1.0),
        Pipe,
        PipeBottom,
        Obstacle,
        Anchor::TOP_CENTER,
    ));
    commands.spawn((
        Sprite::from_atlas_image(
            atlas.texture.clone(),
            TextureAtlas {
                layout: atlas.layout.clone(),
                index: atlas.map["pipe_down"],
            },
        ),
        Transform::from_xyz((SCREEN_WIDTH + 52.0) / 2.0, PIPE_GAP + pipe_offset, 1.0),
        Pipe,
        PipeTop,
        Obstacle,
        Anchor::BOTTOM_CENTER,
    ));
}

#[derive(Component)]
struct Obstacle;

#[derive(Component, Clone, Copy)]
enum CurrentVolume {
    Circle(BoundingCircle),
    Aabb(Aabb2d),
}

#[derive(Component, Deref, DerefMut)]
struct Intersects(pub bool);

fn check_for_collisions(
    mut gizmos: Gizmos,
    time: Res<Time>,
    mut volumes: Query<(&CurrentVolume, &mut Intersects)>,
    players: Query<&Transform, With<Player>>,
    obstacles: Query<
        (
            &Transform,
            Option<&Ground>,
            Option<&Pipe>,
            Option<&PipeTop>,
            Option<&PipeBottom>,
        ),
        With<Obstacle>,
    >,
) {
    let center = get_intersection_position(&time);
    let circle = BoundingCircle::new(center, 50.);
    if DRAW_DEBUG {
        gizmos.circle_2d(center, circle.radius(), YELLOW);
        for player_transform in &players {
            gizmos.circle_2d(
                vec2(
                    player_transform.translation.x,
                    player_transform.translation.y,
                ),
                PLAYER_RADIUS,
                RED,
            );
        }
        for (obstacle_transform, maybe_ground, maybe_pipe, maybe_pipe_top, maybe_pipe_bottom) in
            &obstacles
        {
            let size: Vec2;
            if let Some(_) = maybe_ground {
                size = vec2(GROUND_WIDTH, GROUND_HEIGHT);
            } else if let Some(_) = maybe_pipe {
                size = vec2(PIPE_WIDTH, PIPE_HEIGHT);
            } else {
                continue;
            }
            let position: Vec2;
            if let Some(_) = maybe_ground {
                position = vec2(
                    obstacle_transform.translation.x - GROUND_WIDTH / 2.0,
                    obstacle_transform.translation.y + GROUND_HEIGHT / 2.0,
                );
            } else if let Some(_) = maybe_pipe {
                if let Some(_) = maybe_pipe_top {
                    position = vec2(
                        obstacle_transform.translation.x,
                        obstacle_transform.translation.y + PIPE_HEIGHT / 2.0,
                    );
                } else if let Some(_) = maybe_pipe_bottom {
                    position = vec2(
                        obstacle_transform.translation.x,
                        obstacle_transform.translation.y - PIPE_HEIGHT / 2.0,
                    );
                } else {
                    continue;
                };
            } else {
                continue;
            }
            gizmos.rect_2d(position, size, BLUE);
        }
    }

    for (volume, mut intersects) in volumes.iter_mut() {
        let hit = match volume {
            CurrentVolume::Aabb(a) => circle.intersects(a),
            CurrentVolume::Circle(c) => circle.intersects(c),
        };

        **intersects = hit;
    }
}

fn get_intersection_position(time: &Time) -> Vec2 {
    let x = ops::cos(0.8 * time.elapsed_secs()) * 250.;
    let y = ops::sin(0.4 * time.elapsed_secs()) * 100.;
    Vec2::new(x, y)
}
