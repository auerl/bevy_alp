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

    commands.spawn((
        TiledMap(asset_server.load("ortho-map.tmx")),
        TilemapAnchor::Center,
    ));

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
