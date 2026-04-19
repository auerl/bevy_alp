use bevy::prelude::*;
use bevy_ecs_tiled::prelude::*;

#[derive(Component)]
struct Player {
    speed: f32,
}

#[derive(Component, Clone, Copy, PartialEq, Eq, Default)]
enum Facing {
    Up,
    #[default]
    Down,
    Left,
    Right,
}

impl Facing {
    fn indices(self) -> AnimationIndices {
        match self {
            Facing::Up => AnimationIndices { first: 0, last: 8 },
            Facing::Left => AnimationIndices { first: 9, last: 17 },
            Facing::Down => AnimationIndices { first: 18, last: 26 },
            Facing::Right => AnimationIndices { first: 27, last: 35 },
        }
    }
}

#[derive(Component, Clone, Copy)]
struct AnimationIndices {
    first: usize,
    last: usize,
}

#[derive(Component, Deref, DerefMut)]
struct AnimationTimer(Timer);

#[derive(Component)]
struct Bullet {
    velocity: Vec2,
}

#[derive(Component, Deref, DerefMut)]
struct Lifetime(Timer);

#[derive(Component)]
struct Enemy {
    speed: f32,
}

const BULLET_SPEED: f32 = 600.0;
const BULLET_LIFETIME_SECS: f32 = 1.5;
const BULLET_SPAWN_OFFSET: f32 = 28.0;
const BULLET_HIT_RADIUS: f32 = 4.0;

const ENEMY_SPEED: f32 = 55.0;
const ENEMY_HIT_RADIUS: f32 = 18.0;

const MAP_TILE_W: f32 = 45.0;
const MAP_TILE_H: f32 = 31.0;
const TILE_PX: f32 = 16.0;
const MAP_PX_W: f32 = MAP_TILE_W * TILE_PX;
const MAP_PX_H: f32 = MAP_TILE_H * TILE_PX;

const ENEMY_POSITIONS: &[(f32, f32)] = &[
    (350.0, 220.0),
    (-420.0, 380.0),
    (520.0, -310.0),
    (-640.0, -240.0),
    (760.0, 120.0),
    (-820.0, 460.0),
    (220.0, -540.0),
    (940.0, -100.0),
    (-300.0, 640.0),
    (640.0, 540.0),
    (-720.0, -520.0),
    (420.0, -740.0),
    (-180.0, 180.0),
    (1000.0, 380.0),
    (-960.0, 80.0),
];

const MOVE_KEYS: [KeyCode; 8] = [
    KeyCode::KeyW,
    KeyCode::KeyA,
    KeyCode::KeyS,
    KeyCode::KeyD,
    KeyCode::ArrowUp,
    KeyCode::ArrowDown,
    KeyCode::ArrowLeft,
    KeyCode::ArrowRight,
];

fn main() {
    App::new()
        .insert_resource(ClearColor(Color::srgb(0.0, 0.0, 0.0)))
        .add_plugins(
            DefaultPlugins
                .set(WindowPlugin {
                    primary_window: Some(Window {
                        title: "Alp & Run".into(),
                        ..default()
                    }),
                    ..default()
                })
                .set(ImagePlugin::default_nearest()),
        )
        .add_plugins(TiledPlugin::default())
        .add_systems(Startup, setup)
        .add_systems(
            Update,
            (
                character_input,
                shoot.after(character_input),
                update_bullets,
                enemy_chase.after(character_input),
                bullet_enemy_collision.after(update_bullets).after(enemy_chase),
                animate_character.after(character_input),
                camera_follow.after(character_input),
            ),
        )
        .run();
}

fn setup(
    mut commands: Commands,
    asset_server: Res<AssetServer>,
    mut atlas_layouts: ResMut<Assets<TextureAtlasLayout>>,
) {
    commands.spawn(Camera2d);

    let map_handle = asset_server.load("ortho-map.tmx");
    for gx in -1..=1 {
        for gy in -1..=1 {
            commands.spawn((
                TiledMap(map_handle.clone()),
                TilemapAnchor::Center,
                Transform::from_xyz(gx as f32 * MAP_PX_W, gy as f32 * MAP_PX_H, 0.0),
            ));
        }
    }

    let enemy_texture = asset_server.load("textures/rpg/chars/gabe/gabe-idle-run.png");
    let enemy_layout = atlas_layouts.add(TextureAtlasLayout::from_grid(
        UVec2::splat(24),
        7,
        1,
        None,
        None,
    ));
    for &(x, y) in ENEMY_POSITIONS {
        commands.spawn((
            Sprite::from_atlas_image(
                enemy_texture.clone(),
                TextureAtlas {
                    layout: enemy_layout.clone(),
                    index: 0,
                },
            ),
            Transform::from_xyz(x, y, 5.0).with_scale(Vec3::splat(1.5)),
            Enemy { speed: ENEMY_SPEED },
        ));
    }

    let texture = asset_server.load("textures/rpg/chars/professor_walk_cycle_no_hat.png");
    let layout = atlas_layouts.add(TextureAtlasLayout::from_grid(
        UVec2::splat(64),
        9,
        4,
        None,
        None,
    ));

    let facing = Facing::default();
    let indices = facing.indices();

    commands.spawn((
        Sprite::from_atlas_image(
            texture,
            TextureAtlas {
                layout,
                index: indices.first,
            },
        ),
        Transform::from_xyz(0.0, 0.0, 10.0).with_scale(Vec3::splat(1.5)),
        Player { speed: 220.0 },
        facing,
        indices,
        AnimationTimer(Timer::from_seconds(0.08, TimerMode::Repeating)),
    ));
}

fn character_input(
    time: Res<Time>,
    keyboard: Res<ButtonInput<KeyCode>>,
    mut query: Query<(
        &Player,
        &mut Transform,
        &mut Facing,
        &mut AnimationIndices,
    )>,
) {
    for (player, mut transform, mut facing, mut indices) in &mut query {
        let mut dir = Vec2::ZERO;
        if keyboard.pressed(KeyCode::KeyW) || keyboard.pressed(KeyCode::ArrowUp) {
            dir.y += 1.0;
        }
        if keyboard.pressed(KeyCode::KeyS) || keyboard.pressed(KeyCode::ArrowDown) {
            dir.y -= 1.0;
        }
        if keyboard.pressed(KeyCode::KeyA) || keyboard.pressed(KeyCode::ArrowLeft) {
            dir.x -= 1.0;
        }
        if keyboard.pressed(KeyCode::KeyD) || keyboard.pressed(KeyCode::ArrowRight) {
            dir.x += 1.0;
        }

        if dir == Vec2::ZERO {
            continue;
        }

        let step = dir.normalize() * player.speed * time.delta_secs();
        transform.translation.x += step.x;
        transform.translation.y += step.y;

        let new_facing = if dir.x.abs() > dir.y.abs() {
            if dir.x > 0.0 { Facing::Right } else { Facing::Left }
        } else if dir.y > 0.0 {
            Facing::Up
        } else {
            Facing::Down
        };

        if *facing != new_facing {
            *facing = new_facing;
            *indices = new_facing.indices();
        }
    }
}

fn animate_character(
    time: Res<Time>,
    keyboard: Res<ButtonInput<KeyCode>>,
    mut query: Query<(&AnimationIndices, &mut AnimationTimer, &mut Sprite)>,
) {
    let moving = MOVE_KEYS.iter().any(|k| keyboard.pressed(*k));

    for (indices, mut timer, mut sprite) in &mut query {
        let Some(atlas) = sprite.texture_atlas.as_mut() else {
            continue;
        };

        if !moving {
            atlas.index = indices.first;
            timer.reset();
            continue;
        }

        timer.tick(time.delta());
        if timer.just_finished() {
            atlas.index = if atlas.index >= indices.last || atlas.index < indices.first {
                indices.first
            } else {
                atlas.index + 1
            };
        }
    }
}

fn shoot(
    mut commands: Commands,
    keyboard: Res<ButtonInput<KeyCode>>,
    query: Query<(&Transform, &Facing), With<Player>>,
) {
    if !keyboard.just_pressed(KeyCode::Space) {
        return;
    }
    for (transform, facing) in &query {
        let dir = match facing {
            Facing::Up => Vec2::Y,
            Facing::Down => Vec2::NEG_Y,
            Facing::Left => Vec2::NEG_X,
            Facing::Right => Vec2::X,
        };
        let spawn = transform.translation + (dir * BULLET_SPAWN_OFFSET).extend(0.0);
        commands.spawn((
            Sprite::from_color(Color::srgb(1.0, 0.9, 0.2), Vec2::new(8.0, 8.0)),
            Transform::from_translation(spawn),
            Bullet { velocity: dir * BULLET_SPEED },
            Lifetime(Timer::from_seconds(BULLET_LIFETIME_SECS, TimerMode::Once)),
        ));
    }
}

fn update_bullets(
    time: Res<Time>,
    mut commands: Commands,
    mut query: Query<(Entity, &Bullet, &mut Transform, &mut Lifetime)>,
) {
    for (entity, bullet, mut transform, mut lifetime) in &mut query {
        transform.translation.x += bullet.velocity.x * time.delta_secs();
        transform.translation.y += bullet.velocity.y * time.delta_secs();
        if lifetime.tick(time.delta()).is_finished() {
            commands.entity(entity).despawn();
        }
    }
}

fn enemy_chase(
    time: Res<Time>,
    player: Query<&Transform, (With<Player>, Without<Enemy>)>,
    mut enemies: Query<(&Enemy, &mut Transform), Without<Player>>,
) {
    let Some(p) = player.iter().next() else {
        return;
    };
    for (enemy, mut transform) in &mut enemies {
        let to_player = (p.translation - transform.translation).truncate();
        if to_player.length_squared() < 1.0 {
            continue;
        }
        let dir = to_player.normalize();
        transform.translation.x += dir.x * enemy.speed * time.delta_secs();
        transform.translation.y += dir.y * enemy.speed * time.delta_secs();
    }
}

fn bullet_enemy_collision(
    mut commands: Commands,
    bullets: Query<(Entity, &Transform), (With<Bullet>, Without<Enemy>)>,
    enemies: Query<(Entity, &Transform), (With<Enemy>, Without<Bullet>)>,
) {
    let hit_dist_sq = (ENEMY_HIT_RADIUS + BULLET_HIT_RADIUS).powi(2);
    let mut to_despawn: Vec<Entity> = Vec::new();
    'bullets: for (be, bt) in &bullets {
        for (ee, et) in &enemies {
            if to_despawn.contains(&ee) {
                continue;
            }
            let d = (bt.translation - et.translation).truncate();
            if d.length_squared() < hit_dist_sq {
                to_despawn.push(be);
                to_despawn.push(ee);
                continue 'bullets;
            }
        }
    }
    for e in to_despawn {
        commands.entity(e).despawn();
    }
}

fn camera_follow(
    player: Query<&Transform, (With<Player>, Without<Camera2d>)>,
    mut camera: Query<&mut Transform, (With<Camera2d>, Without<Player>)>,
) {
    let Some(p) = player.iter().next() else {
        return;
    };
    for mut c in &mut camera {
        c.translation.x = p.translation.x;
        c.translation.y = p.translation.y;
    }
}
