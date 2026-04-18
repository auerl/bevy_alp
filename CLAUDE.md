# CLAUDE.md

This file provides guidance to Claude Code (claude.ai/code) when working with code in this repository.

## Commands

- Run: `cargo run` (debug) or `cargo run --release` (much smoother play). **Always run via `cargo` rather than invoking the binary in `target/debug/` directly** — Bevy resolves the `assets/` path relative to `CARGO_MANIFEST_DIR`, so the bare binary cannot find map / sprite assets.
- Type-check only: `cargo check`.
- Format: `cargo fmt` (see `rustfmt.toml`).
- No tests exist.

## Toolchain

Bevy 0.18 requires Rust edition 2024, i.e. **rustc ≥ 1.85**. The project has been verified against 1.95. `rustup update stable` if you land on an older toolchain.

First compile is slow (~15 min debug on a laptop) because of Bevy's dependency graph; incremental rebuilds are fast. `Cargo.toml` sets `opt-level = 3` for transitive dependencies in dev to keep runtime responsive while your own code compiles in debug.

## Dependencies

- `bevy = "0.18"` — game engine.
- `bevy_ecs_tiled = "0.11"` — loads Tiled `.tmx` maps; pulls in `bevy_ecs_tilemap` automatically. `TiledPlugin::default()` must be added alongside `DefaultPlugins`.

The old `bevy_tiled` (StarArawn) and Bevy 0.1.3 dependencies were ripped out in a full rewrite — do not try to revive them, the old unsafe code UB-panics on modern Rust.

## Architecture

Single-binary 2D top-down prototype. All source is in `src/main.rs` (~190 lines). The app wires four systems:

- **`setup` (Startup)** — spawns `Camera2d`, spawns a `TiledMap` entity loading `ortho-map.tmx` with `TilemapAnchor::Center`, builds a `TextureAtlasLayout` (9×4, 64px tiles) for the player sprite, and spawns the player with `Sprite::from_atlas_image`, `Player { speed }`, a `Facing`, an `AnimationIndices` range, and an `AnimationTimer`.
- **`character_input` (Update)** — reads WASD + arrow keys, normalizes to a `Vec2`, moves the player's `Transform`, and updates `Facing` / `AnimationIndices` from the dominant movement axis. Idle (zero input) keeps the last facing.
- **`animate_character` (Update, after input)** — ticks the per-player `AnimationTimer` and advances the `TextureAtlas.index` within the current `AnimationIndices` range. When no movement key is held, it snaps to `indices.first` (idle pose) and resets the timer.
- **`camera_follow` (Update, after input)** — the camera's `Transform.translation` is copied from the player each frame. The camera does **not** have its own keybinds (this fixes a bug from the old code where WASD moved both camera and player simultaneously).

### Spritesheet layout

`assets/textures/rpg/chars/professor_walk_cycle_no_hat.png` is 576×256, laid out as 9 columns × 4 rows of 64×64 frames. The `Facing::indices()` method owns the mapping: rows are `Up 0–8`, `Left 9–17`, `Down 18–26`, `Right 27–35`. If you swap in a different character sheet, update both the `UVec2::splat(64)` grid size in `setup` and the ranges in `Facing::indices`.

### Query disjointness

`camera_follow` uses two `Query<&mut Transform, ...>` — they must stay disjoint. The filters `(With<Player>, Without<Camera2d>)` and `(With<Camera2d>, Without<Player>)` enforce this; removing either `Without` will cause a runtime panic about conflicting access.

## Assets

`assets/` holds the live map (`ortho-map.tmx`) plus its tileset image (`ortho.png`) and several alternate Tiled maps (isometric variants, `mymap.tmx`, etc.) that are not wired up. Character sheets are under `assets/textures/rpg/chars/`.
