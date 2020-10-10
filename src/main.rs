use bevy::{
    ecs::Mut,
    prelude::*,
    render::pass::ClearColor,
    render::camera::Camera,
    sprite::collide_aabb::{collide, Collision},
};

use bevy_tiled;

fn main() {
    App::build()
        .add_resource(WindowDescriptor {
            title: "Alp & Run".to_string(),
            ..Default::default()
        })
        .add_default_plugins()
        .add_resource(ClearColor(Color::rgb(0.0, 0.0, 0.0)))
        .add_plugin(bevy_tiled::TiledMapPlugin)
        .add_startup_system(setup.system())
        .add_system(camera_movement.system())
        //.add_system(paddle_movement_system.system())
        .add_system(character_input_system.system())
        .add_system(character_movement_system.system())
        .add_system(animate_character_system.system())
        .run();
}

struct KeyBinds {
    up: KeyCode,
    down: KeyCode,
    right: KeyCode,
    left: KeyCode,
}

enum Collider {
    Solid,
    Scoreable,
}

enum CharacterMovementState {
    Up,
    Down,
    Right,
    Left,
    UpLeft,
    UpRight,
    DownLeft,
    DownRight,
    None,
}

struct Character {
    move_state: CharacterMovementState,
    speed: f32,
}

struct Player {
    binds: KeyBinds,
}

fn setup(
    mut commands: Commands,
    mut materials: ResMut<Assets<ColorMaterial>>,
    asset_server: Res<AssetServer>,
    mut textures: ResMut<Assets<Texture>>,
    mut texture_atlases: ResMut<Assets<TextureAtlas>>,
) {
    let texture_handle = asset_server
        .load_sync(
            &mut textures,
            // "assets/textures/rpg/chars/gabe/gabe-idle-run.png",
            "assets/textures/rpg/chars/professor_walk_cycle_no_hat.png",
        )
        .unwrap();
    let texture = textures.get(&texture_handle).unwrap();
    let texture_atlas = TextureAtlas::from_grid(texture_handle, texture.size, 9, 4); //7, 1);
    let texture_atlas_handle = texture_atlases.add(texture_atlas);

    commands
        .spawn(bevy_tiled::TiledMapComponents {
            map_asset: asset_server.load("assets/ortho-map.tmx").unwrap(),
            center: true,
            ..Default::default()
        })

        // .spawn(bevy_tiled::TiledMapComponents {
        //     map_asset: asset_server.load("assets/isometric_grass_and_water.tmx").unwrap(),
        //     center: true,
        //     ..Default::default()
        // })
        .spawn(Camera2dComponents::default());



    commands
        .spawn(Camera2dComponents::default())
        .spawn(UiCameraComponents::default())
        .spawn(SpriteSheetComponents {
            texture_atlas: texture_atlas_handle,
            translation: Translation(Vec3::new(0.0, 0.0, 2.5)),
            scale: Scale(2.0),
            draw: Draw {
                is_transparent: true,
                ..Default::default()
            },
            ..Default::default()
        })
        .with(Player {
            binds: KeyBinds {
                up: KeyCode::W,
                down: KeyCode::S,
                left: KeyCode::A,
                right: KeyCode::D,
            },
        })
        .with(Character {
            move_state: CharacterMovementState::None,
            speed: 400.0,
        })
        .with(Collider::Solid)
        .with(Timer::from_seconds(0.05, true));

}

fn animate_character_system(
    texture_atlases: Res<Assets<TextureAtlas>>,
    mut query: Query<(&mut Timer, &Character, &mut TextureAtlasSprite, &Handle<TextureAtlas>)>,
) {
    for (mut timer, character, mut sprite, texture_atlas_handle) in &mut query.iter() {
        if timer.finished {
            timer.reset();
            match character.move_state {
                CharacterMovementState::Right | CharacterMovementState::DownRight | CharacterMovementState::UpRight => {
                    sprite.index = if sprite.index < 35 && sprite.index >= 27 {sprite.index + 1} else {27}
                }
                CharacterMovementState::Down => {
                    sprite.index = if sprite.index < 26 && sprite.index >= 18 {sprite.index + 1} else {18}
                }
                CharacterMovementState::Left | CharacterMovementState::DownLeft | CharacterMovementState::UpLeft => {
                    sprite.index = if sprite.index < 17 && sprite.index >= 9 {sprite.index + 1} else {9}
                }
                CharacterMovementState::Up => {
                    sprite.index = if sprite.index < 8 {sprite.index + 1} else {0}
                }
                CharacterMovementState::None => {
                    sprite.index = if sprite.index < 8 {
                        0
                    } else if sprite.index >= 9 && sprite.index < 17 {
                        9
                    } else if sprite.index >= 18 && sprite.index < 26 {
                        18
                    } else if sprite.index >= 27 && sprite.index < 35 {
                        27
                    } else {
                        sprite.index
                    }
                }
                _ => {}
            }
        }
    }
}


fn character_input_system(
    keyboard_input: Res<Input<KeyCode>>,
    mut query: Query<(&Player, &mut Character)>,
) {
    for (player, mut character) in &mut query.iter() {
        if keyboard_input.pressed(player.binds.up) && keyboard_input.pressed(player.binds.left) {
            character.move_state = CharacterMovementState::UpLeft
        } else if keyboard_input.pressed(player.binds.down) && keyboard_input.pressed(player.binds.left) {
            character.move_state = CharacterMovementState::DownLeft
        } else if keyboard_input.pressed(player.binds.up) && keyboard_input.pressed(player.binds.right) {
            character.move_state = CharacterMovementState::UpRight
        } else if keyboard_input.pressed(player.binds.down) && keyboard_input.pressed(player.binds.right) {
            character.move_state = CharacterMovementState::DownRight
        } else if keyboard_input.pressed(player.binds.up) {
            character.move_state = CharacterMovementState::Up
        } else if keyboard_input.pressed(player.binds.down) {
            character.move_state = CharacterMovementState::Down
        } else if keyboard_input.pressed(player.binds.left) {
            character.move_state = CharacterMovementState::Left
        } else if keyboard_input.pressed(player.binds.right) {
            character.move_state = CharacterMovementState::Right;
        } else {
            character.move_state = CharacterMovementState::None
        }

    }
}


fn camera_movement(
    time: Res<Time>,
    keyboard_input: Res<Input<KeyCode>>,
    mut query: Query<(&Camera, &mut Translation)>,
) {
    for (_, mut translation) in &mut query.iter() {
        let mut direction = Vec3::zero();
        if keyboard_input.pressed(KeyCode::A) {
            direction -= Vec3::new(1.0, 0.0, 0.0);
        }

        if keyboard_input.pressed(KeyCode::D) {
            direction +=  Vec3::new(1.0, 0.0, 0.0);
        }

        if keyboard_input.pressed(KeyCode::W) {
            direction += Vec3::new(0.0, 1.0, 0.0);
        }

        if keyboard_input.pressed(KeyCode::S) {
            direction -=  Vec3::new(0.0, 1.0, 0.0);
        }

        translation.0 += time.delta_seconds * direction * 1000.0;
    }
}

fn character_movement_system(time: Res<Time>, keyboard_input: Res<Input<KeyCode>>, mut query: Query<(&Character, &mut Translation)>) {
    for (_, mut translation) in &mut query.iter() {
        let mut direction = Vec3::zero();

        if keyboard_input.pressed(KeyCode::A) {
            direction -= Vec3::new(1.0, 0.0, 0.0);
        }

        if keyboard_input.pressed(KeyCode::D) {
            direction +=  Vec3::new(1.0, 0.0, 0.0);
        }

        if keyboard_input.pressed(KeyCode::W) {
            direction += Vec3::new(0.0, 1.0, 0.0);
        }

        if keyboard_input.pressed(KeyCode::S) {
            direction -=  Vec3::new(0.0, 1.0, 0.0);
        }

        translation.0 += time.delta_seconds * direction * 1000.0;
    }
}


