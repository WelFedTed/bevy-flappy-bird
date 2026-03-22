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
const PIPE_GAP: f32 = 55.0; // distance between pipes vertically for bird to pass through
const PIPE_INTERVAL: f32 = 175.0; // distance between pipe pairs horizontally
const GROUND_WIDTH: f32 = 336.0;
const GROUND_HEIGHT: f32 = 112.0;
const PIPE_WIDTH: f32 = 52.0;
const PIPE_HEIGHT: f32 = 320.0;
const PLAYER_COLLIDER_RADIUS: f32 = 10.0; // radius of the player's collision circle
const DRAW_DEBUG: bool = false; // toggle to draw debug gizmos for collision detection
const INVINCIBLE: bool = false; // toggle player invincibility
const DIE_SOUND_DELAY: f32 = 0.5; // delay before playing the die sound after collision
const FLASH_DURATION: f32 = 0.075; // duration of the screen flash after collision

static mut DEAD: bool = false;
// static mut SCORE: u32 = 0;

#[derive(Resource)]
struct Score(u32);

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
        .insert_resource(Score(0))
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
        .add_systems(PostUpdate, render_volumes)
        .add_systems(Update, play_die_sound_after_delay)
        .add_systems(Update, update_screen_flash)
        .add_systems(Update, mark_passed_pipes)
        .add_systems(Update, update)
        .run();
}

fn update(score: ResMut<Score>) {
    println!("Score: {}", score.0);
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
        // CurrentVolume::Circle(BoundingCircle::new(Vec2::new(-50.0, 0.0), PLAYER_COLLIDER_RADIUS)),
        // Intersects(false),
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
    if keyboard.just_pressed(KeyCode::Space) && !unsafe { DEAD } {
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
        Transform::from_xyz(0.0, -SCREEN_HEIGHT / 2.0, 2.0),
        Anchor::BOTTOM_CENTER,
        Ground,
        Obstacle,
        CurrentVolume::Aabb(Aabb2d::new(
            Vec2::new(0.0, GROUND_HEIGHT / 2.0 - SCREEN_HEIGHT / 2.0),
            Vec2::new(0.0, GROUND_HEIGHT / 2.0),
        )),
        Intersects(false),
    ));
    spawn_next_ground(&mut commands, &atlas);
}

fn move_obstacles(
    time: Res<Time>,
    mut obstacles: Query<
        (
            &mut Transform,
            &mut CurrentVolume,
            Option<&Ground>,
            Option<&Pipe>,
        ),
        With<Obstacle>,
    >,
) {
    if !unsafe { DEAD } {
        for (mut transform, mut volume, maybe_ground, maybe_pipe) in &mut obstacles {
            transform.translation.x -= SCROLL_SPEED * time.delta_secs();
            match &mut *volume {
                CurrentVolume::Aabb(aabb) => {
                    if let Some(_ground) = maybe_ground {
                        aabb.min.x = transform.translation.x - GROUND_WIDTH / 2.0;
                        aabb.max.x = transform.translation.x + GROUND_WIDTH / 2.0;
                    } else if let Some(_pipe) = maybe_pipe {
                        aabb.min.x = transform.translation.x - PIPE_WIDTH / 2.0;
                        aabb.max.x = transform.translation.x + PIPE_WIDTH / 2.0;
                    }
                } // CurrentVolume::Circle(circle) => {
                  //     circle.center = vec2(transform.translation.x, transform.translation.y);
                  // }
            }
        }
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
        Transform::from_xyz(
            SCREEN_WIDTH / 2.0 + GROUND_WIDTH / 2.0,
            -SCREEN_HEIGHT / 2.0,
            2.0,
        ),
        Ground,
        Obstacle,
        CurrentVolume::Aabb(Aabb2d::new(
            Vec2::new(0.0, GROUND_HEIGHT / 2.0 - SCREEN_HEIGHT / 2.0),
            Vec2::new(0.0, GROUND_HEIGHT / 2.0),
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
            if transform.translation.x <= (-SCREEN_WIDTH - GROUND_WIDTH) / 2.0 {
                commands.entity(entity).despawn();
                // println!("REMOVED GROUND");
                spawn_next_ground(&mut commands, &atlas);
            }
        }
        if let Some(_) = maybe_pipe {
            if transform.translation.x <= -(SCREEN_WIDTH + PIPE_WIDTH) / 2.0 {
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

#[derive(Component)]
struct Passed;

fn mark_passed_pipes(
    mut commands: Commands,
    mut query: Query<(Entity, &Transform), (With<PipeTop>, Without<Passed>)>,
    player_query: Query<&Transform, With<Player>>,
    mut score: ResMut<Score>,
    asset_server: Res<AssetServer>,
) {
    let player_x = player_query.single().unwrap().translation.x;

    for (entity, transform) in &mut query {
        if transform.translation.x < player_x + PIPE_WIDTH / 4.0 {
            commands.entity(entity).insert(Passed);
            let audio = asset_server.load("sfx_point.ogg");
            commands.spawn((AudioPlayer::new(audio), PlaybackSettings::ONCE));
            score.0 += 1;
        }
    }
}

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
                (SCREEN_WIDTH + PIPE_WIDTH) / 2.0 + PIPE_INTERVAL * i as f32,
                -PIPE_GAP + pipe_offset,
                1.0,
            ),
            Pipe,
            PipeBottom,
            Obstacle,
            Passed,
            CurrentVolume::Aabb(Aabb2d::new(
                Vec2::new(0.0, pipe_offset - PIPE_GAP - PIPE_HEIGHT),
                Vec2::new(0.0, PIPE_HEIGHT),
            )),
            Intersects(false),
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
                (SCREEN_WIDTH + PIPE_WIDTH) / 2.0 + PIPE_INTERVAL * i as f32,
                PIPE_GAP + pipe_offset,
                1.0,
            ),
            Pipe,
            PipeTop,
            Obstacle,
            CurrentVolume::Aabb(Aabb2d::new(
                Vec2::new(0.0, pipe_offset + PIPE_GAP + PIPE_HEIGHT),
                Vec2::new(0.0, PIPE_HEIGHT),
            )),
            Intersects(false),
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
        Transform::from_xyz(
            (SCREEN_WIDTH + PIPE_WIDTH) / 2.0,
            -PIPE_GAP + pipe_offset,
            1.0,
        ),
        Pipe,
        PipeBottom,
        Obstacle,
        CurrentVolume::Aabb(Aabb2d::new(
            Vec2::new(0.0, pipe_offset - PIPE_GAP - PIPE_HEIGHT),
            Vec2::new(0.0, PIPE_HEIGHT),
        )),
        Intersects(false),
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
            (SCREEN_WIDTH + PIPE_WIDTH) / 2.0,
            PIPE_GAP + pipe_offset,
            1.0,
        ),
        Pipe,
        PipeTop,
        Obstacle,
        CurrentVolume::Aabb(Aabb2d::new(
            Vec2::new(0.0, pipe_offset + PIPE_GAP + PIPE_HEIGHT),
            Vec2::new(0.0, PIPE_HEIGHT),
        )),
        Intersects(false),
        Anchor::BOTTOM_CENTER,
    ));
}

#[derive(Component)]
struct Obstacle;

#[derive(Component, Clone, Copy)]
enum CurrentVolume {
    // Circle(BoundingCircle),
    Aabb(Aabb2d),
}

#[derive(Component, Deref, DerefMut)]
struct Intersects(pub bool);

fn check_for_collisions(
    mut gizmos: Gizmos,
    mut volumes: Query<(&CurrentVolume, &mut Intersects)>,
    players: Query<&Transform, With<Player>>,
    asset_server: Res<AssetServer>,
    mut commands: Commands,
) {
    let mut circle = BoundingCircle::new(Vec2::ZERO, 0.0);
    for player_transform in &players {
        circle = BoundingCircle::new(
            vec2(
                player_transform.translation.x,
                player_transform.translation.y,
            ),
            PLAYER_COLLIDER_RADIUS,
        )
    }
    if DRAW_DEBUG {
        gizmos.circle_2d(circle.center, circle.radius(), YELLOW);
    }

    for (volume, mut intersects) in volumes.iter_mut() {
        let hit = match volume {
            CurrentVolume::Aabb(a) => circle.intersects(a),
            // CurrentVolume::Circle(c) => circle.intersects(c),
        };

        **intersects = hit;

        if hit && !INVINCIBLE {
            // println!("HIT {:?}", circle.center);
            if !unsafe { DEAD } {
                trigger_screen_flash(&mut commands);
                let audio = asset_server.load("sfx_hit.ogg");
                commands.spawn((AudioPlayer::new(audio), PlaybackSettings::ONCE));
                // wait 1 second before playing the die sound
                commands.spawn(PlaySoundAfterDelay(Timer::from_seconds(
                    DIE_SOUND_DELAY,
                    TimerMode::Once,
                )));
                unsafe { DEAD = true };
            }
        }
    }
}

#[derive(Component)]
struct PlaySoundAfterDelay(Timer);

fn play_die_sound_after_delay(
    mut commands: Commands,
    time: Res<Time>,
    asset_server: Res<AssetServer>,
    mut query: Query<(Entity, &mut PlaySoundAfterDelay)>,
) {
    for (entity, mut delay) in &mut query {
        delay.0.tick(time.delta());

        if delay.0.is_finished() {
            let audio = asset_server.load("sfx_die.ogg");
            commands.spawn((AudioPlayer::new(audio), PlaybackSettings::ONCE));

            // remove the timer entity
            commands.entity(entity).despawn();
        }
    }
}

fn render_volumes(mut gizmos: Gizmos, query: Query<(&CurrentVolume, &Intersects)>) {
    if DRAW_DEBUG {
        for (volume, intersects) in query.iter() {
            let color = if **intersects { AQUA } else { ORANGE_RED };
            match volume {
                CurrentVolume::Aabb(a) => {
                    gizmos.rect_2d(a.center(), a.half_size() * 2.0, color);
                } // CurrentVolume::Circle(c) => {
                  //     gizmos.circle_2d(c.center(), c.radius(), color);
                  // }
            }
        }
    }
}

#[derive(Component)]
struct ScreenFlash {
    timer: Timer,
}

fn trigger_screen_flash(commands: &mut Commands) {
    commands.spawn((
        Node {
            width: Val::Percent(100.0),
            height: Val::Percent(100.0),
            position_type: PositionType::Absolute,
            ..default()
        },
        BackgroundColor(Color::WHITE),
        // ZIndex::Global(999), // ensure it's on top
        ScreenFlash {
            timer: Timer::from_seconds(FLASH_DURATION, TimerMode::Once), // total flash duration
        },
    ));
}

fn update_screen_flash(
    mut commands: Commands,
    time: Res<Time>,
    mut query: Query<(Entity, &mut ScreenFlash, &mut BackgroundColor)>,
) {
    for (entity, mut flash, mut color) in &mut query {
        flash.timer.tick(time.delta());

        let remaining = 1.0 - flash.timer.fraction();
        color.0.set_alpha(remaining); // fade alpha

        if flash.timer.is_finished() {
            commands.entity(entity).despawn();
        }
    }
}
