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
            Facing::Down => AnimationIndices {
                first: 18,
                last: 26,
            },
            Facing::Right => AnimationIndices {
                first: 27,
                last: 35,
            },
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
    damage: f32,
}

#[derive(Component, Deref, DerefMut)]
struct Lifetime(Timer);

#[derive(Component)]
struct Enemy {
    speed: f32,
}

#[derive(Component, Deref, DerefMut)]
struct EnemyAnimTimer(Timer);

#[derive(Component)]
struct MuzzleFlash;

#[derive(Resource, Default)]
struct ScreenShake {
    trauma: f32,
}

#[derive(Component)]
struct Health {
    current: f32,
    max: f32,
}

#[derive(Component, Deref, DerefMut)]
struct InvulnTimer(Timer);

#[derive(Component)]
struct HealthBarFill;

#[derive(Component)]
struct ScoreText;

#[derive(Component)]
struct GameOverOverlay;

#[derive(Resource, Default)]
struct Score(u32);

#[derive(Resource, Default, PartialEq, Eq, Clone, Copy)]
enum GameState {
    #[default]
    Playing,
    GameOver,
}

const BULLET_SPEED: f32 = 600.0;
const BULLET_LIFETIME_SECS: f32 = 1.5;
const BULLET_SPAWN_OFFSET: f32 = 28.0;
const BULLET_HIT_RADIUS: f32 = 4.0;
const BULLET_DAMAGE: f32 = 15.0;

const ENEMY_SPEED: f32 = 55.0;
const ENEMY_HIT_RADIUS: f32 = 18.0;
const ENEMY_MAX_HP: f32 = 30.0;
const ENEMY_TOUCH_DAMAGE: f32 = 10.0;
const ENEMY_TOUCH_RADIUS: f32 = 22.0;

const PLAYER_MAX_HP: f32 = 100.0;
const PLAYER_INVULN_SECS: f32 = 0.6;
const SCORE_PER_KILL: u32 = 10;

const HEALTH_BAR_WIDTH: f32 = 220.0;
const HEALTH_BAR_HEIGHT: f32 = 18.0;

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
        .insert_resource(Score::default())
        .insert_resource(GameState::default())
        .insert_resource(ScreenShake::default())
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
        .add_systems(Startup, (setup, setup_hud))
        .add_systems(
            Update,
            (
                character_input,
                shoot.after(character_input),
                update_bullets,
                despawn_lifetimes.after(update_bullets),
                enemy_chase.after(character_input),
                animate_enemies,
                bullet_enemy_collision
                    .after(update_bullets)
                    .after(enemy_chase),
                enemy_player_contact.after(enemy_chase),
                tick_invuln,
                check_game_over.after(enemy_player_contact),
                update_hud
                    .after(bullet_enemy_collision)
                    .after(enemy_player_contact),
                restart_on_key,
                animate_character.after(character_input),
                decay_screen_shake,
                camera_follow
                    .after(character_input)
                    .after(decay_screen_shake),
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
            Health {
                current: ENEMY_MAX_HP,
                max: ENEMY_MAX_HP,
            },
            EnemyAnimTimer(Timer::from_seconds(0.10, TimerMode::Repeating)),
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
        Health {
            current: PLAYER_MAX_HP,
            max: PLAYER_MAX_HP,
        },
        InvulnTimer(Timer::from_seconds(0.0, TimerMode::Once)),
    ));
}

fn character_input(
    time: Res<Time>,
    keyboard: Res<ButtonInput<KeyCode>>,
    state: Res<GameState>,
    mut query: Query<(&Player, &mut Transform, &mut Facing, &mut AnimationIndices)>,
) {
    if *state != GameState::Playing {
        return;
    }
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
            if dir.x > 0.0 {
                Facing::Right
            } else {
                Facing::Left
            }
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
    state: Res<GameState>,
    mut shake: ResMut<ScreenShake>,
    query: Query<(&Transform, &Facing), With<Player>>,
) {
    if *state != GameState::Playing {
        return;
    }
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
            Bullet {
                velocity: dir * BULLET_SPEED,
                damage: BULLET_DAMAGE,
            },
            Lifetime(Timer::from_seconds(BULLET_LIFETIME_SECS, TimerMode::Once)),
        ));
        commands.spawn((
            Sprite::from_color(Color::srgb(1.0, 1.0, 0.7), Vec2::new(16.0, 16.0)),
            Transform::from_translation(spawn),
            Lifetime(Timer::from_seconds(0.06, TimerMode::Once)),
            MuzzleFlash,
        ));
        shake.trauma = (shake.trauma + 0.4).min(1.0);
    }
}

fn update_bullets(time: Res<Time>, mut query: Query<(&Bullet, &mut Transform)>) {
    for (bullet, mut transform) in &mut query {
        transform.translation.x += bullet.velocity.x * time.delta_secs();
        transform.translation.y += bullet.velocity.y * time.delta_secs();
    }
}

fn despawn_lifetimes(
    time: Res<Time>,
    mut commands: Commands,
    mut query: Query<(Entity, &mut Lifetime)>,
) {
    for (entity, mut lifetime) in &mut query {
        if lifetime.tick(time.delta()).is_finished() {
            commands.entity(entity).despawn();
        }
    }
}

fn animate_enemies(
    time: Res<Time>,
    mut query: Query<(&mut EnemyAnimTimer, &mut Sprite), With<Enemy>>,
) {
    for (mut timer, mut sprite) in &mut query {
        let Some(atlas) = sprite.texture_atlas.as_mut() else {
            continue;
        };
        if timer.tick(time.delta()).just_finished() {
            atlas.index = (atlas.index + 1) % 7;
        }
    }
}

fn decay_screen_shake(time: Res<Time>, mut shake: ResMut<ScreenShake>) {
    let dt = time.delta_secs();
    shake.trauma = (shake.trauma - 1.4 * dt).max(0.0);
}

fn enemy_chase(
    time: Res<Time>,
    state: Res<GameState>,
    player: Query<&Transform, (With<Player>, Without<Enemy>)>,
    mut enemies: Query<(&Enemy, &mut Transform), Without<Player>>,
) {
    if *state != GameState::Playing {
        return;
    }
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
    mut score: ResMut<Score>,
    bullets: Query<(Entity, &Transform, &Bullet), Without<Enemy>>,
    mut enemies: Query<(Entity, &Transform, &mut Health), (With<Enemy>, Without<Bullet>)>,
) {
    let hit_dist_sq = (ENEMY_HIT_RADIUS + BULLET_HIT_RADIUS).powi(2);
    let mut spent_bullets: Vec<Entity> = Vec::new();
    'bullets: for (be, bt, bullet) in &bullets {
        for (ee, et, mut hp) in &mut enemies {
            if hp.current <= 0.0 {
                continue;
            }
            let d = (bt.translation - et.translation).truncate();
            if d.length_squared() < hit_dist_sq {
                hp.current -= bullet.damage;
                spent_bullets.push(be);
                if hp.current <= 0.0 {
                    score.0 += SCORE_PER_KILL;
                    commands.entity(ee).despawn();
                }
                continue 'bullets;
            }
        }
    }
    for e in spent_bullets {
        commands.entity(e).despawn();
    }
}

fn enemy_player_contact(
    state: Res<GameState>,
    enemies: Query<&Transform, (With<Enemy>, Without<Player>)>,
    mut player: Query<(&Transform, &mut Health, &mut InvulnTimer), With<Player>>,
) {
    if *state != GameState::Playing {
        return;
    }
    let Ok((pt, mut hp, mut invuln)) = player.single_mut() else {
        return;
    };
    if !invuln.is_finished() {
        return;
    }
    let touch_sq = (ENEMY_TOUCH_RADIUS + 12.0).powi(2);
    for et in &enemies {
        let d = (pt.translation - et.translation).truncate();
        if d.length_squared() < touch_sq {
            hp.current = (hp.current - ENEMY_TOUCH_DAMAGE).max(0.0);
            invuln.set_duration(std::time::Duration::from_secs_f32(PLAYER_INVULN_SECS));
            invuln.reset();
            break;
        }
    }
}

fn tick_invuln(time: Res<Time>, mut q: Query<&mut InvulnTimer>) {
    for mut t in &mut q {
        t.tick(time.delta());
    }
}

fn check_game_over(
    mut state: ResMut<GameState>,
    player: Query<&Health, With<Player>>,
    mut overlay: Query<&mut Node, With<GameOverOverlay>>,
) {
    let Ok(hp) = player.single() else { return };
    let should_be_over = hp.current <= 0.0;
    let now_over = *state == GameState::GameOver;
    if should_be_over && !now_over {
        *state = GameState::GameOver;
        for mut node in &mut overlay {
            node.display = Display::Flex;
        }
    }
}

fn setup_hud(mut commands: Commands) {
    commands
        .spawn(Node {
            position_type: PositionType::Absolute,
            top: Val::Px(12.0),
            left: Val::Px(12.0),
            flex_direction: FlexDirection::Column,
            row_gap: Val::Px(6.0),
            ..default()
        })
        .with_children(|root| {
            root.spawn((
                Text::new("Score: 0"),
                TextFont {
                    font_size: 22.0,
                    ..default()
                },
                TextColor(Color::srgb(1.0, 1.0, 1.0)),
                ScoreText,
            ));
            root.spawn((
                Node {
                    width: Val::Px(HEALTH_BAR_WIDTH),
                    height: Val::Px(HEALTH_BAR_HEIGHT),
                    border: UiRect::all(Val::Px(2.0)),
                    ..default()
                },
                BorderColor::all(Color::srgb(0.9, 0.9, 0.9)),
                BackgroundColor(Color::srgb(0.15, 0.05, 0.05)),
            ))
            .with_children(|bar| {
                bar.spawn((
                    Node {
                        width: Val::Px(HEALTH_BAR_WIDTH - 4.0),
                        height: Val::Px(HEALTH_BAR_HEIGHT - 4.0),
                        ..default()
                    },
                    BackgroundColor(Color::srgb(0.85, 0.15, 0.15)),
                    HealthBarFill,
                ));
            });
        });

    commands
        .spawn((
            Node {
                position_type: PositionType::Absolute,
                top: Val::Px(0.0),
                left: Val::Px(0.0),
                width: Val::Percent(100.0),
                height: Val::Percent(100.0),
                display: Display::None,
                justify_content: JustifyContent::Center,
                align_items: AlignItems::Center,
                ..default()
            },
            BackgroundColor(Color::srgba(0.0, 0.0, 0.0, 0.65)),
            GameOverOverlay,
        ))
        .with_children(|root| {
            root.spawn((
                Text::new("GAME OVER\nPress R to restart"),
                TextFont {
                    font_size: 48.0,
                    ..default()
                },
                TextColor(Color::srgb(1.0, 0.95, 0.85)),
                TextLayout::new_with_justify(Justify::Center),
            ));
        });
}

fn update_hud(
    score: Res<Score>,
    player: Query<&Health, With<Player>>,
    mut score_text: Query<&mut Text, With<ScoreText>>,
    mut bar: Query<&mut Node, With<HealthBarFill>>,
) {
    if let Ok(mut t) = score_text.single_mut() {
        **t = format!("Score: {}", score.0);
    }
    let Ok(hp) = player.single() else { return };
    let frac = (hp.current / hp.max).clamp(0.0, 1.0);
    if let Ok(mut node) = bar.single_mut() {
        node.width = Val::Px((HEALTH_BAR_WIDTH - 4.0) * frac);
    }
}

fn restart_on_key(
    mut commands: Commands,
    keyboard: Res<ButtonInput<KeyCode>>,
    mut state: ResMut<GameState>,
    mut score: ResMut<Score>,
    asset_server: Res<AssetServer>,
    mut atlas_layouts: ResMut<Assets<TextureAtlasLayout>>,
    mut player: Query<
        (
            &mut Health,
            &mut Transform,
            &mut InvulnTimer,
            &mut Facing,
            &mut AnimationIndices,
        ),
        With<Player>,
    >,
    enemies: Query<Entity, With<Enemy>>,
    bullets: Query<Entity, With<Bullet>>,
    mut overlay: Query<&mut Node, With<GameOverOverlay>>,
) {
    if *state != GameState::GameOver || !keyboard.just_pressed(KeyCode::KeyR) {
        return;
    }

    for e in &enemies {
        commands.entity(e).despawn();
    }
    for e in &bullets {
        commands.entity(e).despawn();
    }

    if let Ok((mut hp, mut tf, mut invuln, mut facing, mut indices)) = player.single_mut() {
        hp.current = hp.max;
        tf.translation.x = 0.0;
        tf.translation.y = 0.0;
        invuln.set_duration(std::time::Duration::from_secs_f32(0.0));
        invuln.reset();
        *facing = Facing::default();
        *indices = facing.indices();
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
            Health {
                current: ENEMY_MAX_HP,
                max: ENEMY_MAX_HP,
            },
            EnemyAnimTimer(Timer::from_seconds(0.10, TimerMode::Repeating)),
        ));
    }

    score.0 = 0;
    *state = GameState::Playing;
    for mut node in &mut overlay {
        node.display = Display::None;
    }
}

fn camera_follow(
    time: Res<Time>,
    shake: Res<ScreenShake>,
    player: Query<&Transform, (With<Player>, Without<Camera2d>)>,
    mut camera: Query<&mut Transform, (With<Camera2d>, Without<Player>)>,
) {
    let Some(p) = player.iter().next() else {
        return;
    };
    let mag = shake.trauma * shake.trauma * 6.0;
    let t = time.elapsed_secs();
    let ox = (t * 47.3).sin() * mag;
    let oy = (t * 38.7 + 1.7).cos() * mag;
    for mut c in &mut camera {
        c.translation.x = p.translation.x + ox;
        c.translation.y = p.translation.y + oy;
    }
}
