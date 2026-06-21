use rand::rngs::StdRng;
use rand::{Rng, SeedableRng};
use std::env;
use std::sync::{Arc, Mutex};

#[derive(PartialEq)]
pub enum AppPhase {
    StartScreen,
    Playing,
    GameOver,
}

pub struct PlayerStats {
    pub health: u8,
    pub armor: u8,
    pub ap: u8,
    pub is_supercharging: bool,
    pub has_shield: bool,
    pub active_item: u8,
    pub item_charges: u8,
}

pub struct GameState {
    pub phase: AppPhase,
    pub stats: PlayerStats,
    pub map_matrix: Vec<u8>,
    pub width: usize,
    pub height: usize,
    pub seed: u64,
    pub player_idx: usize,
    pub feedback: String,
}

pub type SharedState = Arc<Mutex<GameState>>;

#[derive(Clone, Copy, PartialEq, Eq, Debug)]
pub enum Tile {
    Empty = 0x00,
    Wall = 0x01,
    Player = 0x0A,
    Enemy = 0x0B,
    Health = 0x10,
    Smoke = 0x12,
    Mine = 0x13,
    Resource = 0x14,
    Wreck = 0x11,
}

/// Generate a deterministic tile map from a seed and dimensions.
fn generate_map(seed: u64, width: usize, height: usize) -> (Vec<u8>, usize) {
    let mut rng = StdRng::seed_from_u64(seed);
    let size = width * height;
    let mut map = vec![Tile::Empty as u8; size];
    let mut player_idx = 0;

    // Place walls randomly (~12% of tiles)
    for item in map.iter_mut().take(size) {
        if rng.gen_bool(0.12) {
            *item = Tile::Wall as u8;
        }
    }

    // Place resource nodes (~3%) and pickups (~4%)
    for item in map.iter_mut().take(size) {
        if *item == Tile::Empty as u8 {
            let roll: f64 = rng.gen_range(0.0..1.0);
            if roll < 0.03 {
                *item = Tile::Resource as u8;
            } else if roll < 0.07 {
                // health or other pickups
                *item = if rng.gen_bool(0.5) {
                    Tile::Health as u8
                } else {
                    Tile::Smoke as u8
                };
            }
        }
    }

    // Ensure at least one player and one enemy placed
    let mut placed = 0;
    for (i, item) in map.iter_mut().take(size).enumerate() {
        if *item == Tile::Empty as u8 {
            if placed == 0 {
                *item = Tile::Player as u8;
                player_idx = i;
            } else if placed == 1 {
                *item = Tile::Enemy as u8;
            }
            placed += 1;
            if placed >= 2 {
                break;
            }
        }
    }

    (map, player_idx)
}

/// Create the initial shared game state, respecting optional environment overrides.
pub fn initialize_state() -> SharedState {
    // Allow overriding seed and size via env vars for replayability
    let seed = env::var("GRID_SEED")
        .ok()
        .and_then(|s| s.parse::<u64>().ok())
        .unwrap_or_else(|| {
            // fallback to time-based seed
            use std::time::{SystemTime, UNIX_EPOCH};
            SystemTime::now()
                .duration_since(UNIX_EPOCH)
                .map(|d| d.as_secs())
                .unwrap_or(0)
        });

    let size = env::var("GRID_SIZE")
        .ok()
        .and_then(|s| s.parse::<usize>().ok())
        .unwrap_or(8usize);
    let width = size;
    let height = size;

    let (map, player_idx) = generate_map(seed, width, height);

    let initial_state = GameState {
        phase: AppPhase::StartScreen,
        stats: PlayerStats {
            health: 100,
            armor: 50,
            ap: 12,
            is_supercharging: false,
            has_shield: false,
            active_item: 0,
            item_charges: 0,
        },
        map_matrix: map,
        width,
        height,
        seed,
        player_idx,
        feedback: "Neural link established. Awaiting input.".to_string(),
    };

    Arc::new(Mutex::new(initial_state))
}

/// Regenerate the map for an existing GameState with a new seed and size.
/// Replace the current map with a new seeded map and resize the playfield.
pub fn regenerate_map(state: &mut GameState, seed: u64, size: usize) {
    let (new_map, new_player_idx) = generate_map(seed, size, size);
    state.map_matrix = new_map;
    state.width = size;
    state.height = size;
    state.seed = seed;
    state.player_idx = new_player_idx;
    state.feedback = "Map regenerated.".to_string();
}

/// Move the player by dx,dy if there is enough AP and no wall. Returns true if moved.
/// Move the player by a delta if the destination is valid and AP is available.
pub fn move_player(state: &mut GameState, dx: isize, dy: isize) -> bool {
    let idx = state.player_idx;
    let x = idx % state.width;
    let y = idx / state.width;
    let nx = x as isize + dx;
    let ny = y as isize + dy;
    if nx < 0 || ny < 0 || nx >= state.width as isize || ny >= state.height as isize {
        state.feedback = "Cannot move outside map boundaries.".to_string();
        return false;
    }
    let nidx = (ny as usize) * state.width + (nx as usize);
    if state.map_matrix[nidx] == Tile::Wall as u8 || state.map_matrix[nidx] == Tile::Enemy as u8 {
        state.feedback = "Path blocked.".to_string();
        return false;
    }
    if state.stats.ap == 0 {
        state.feedback = "Not enough AP to move.".to_string();
        return false;
    }
    state.stats.ap = state.stats.ap.saturating_sub(1);
    let target_tile = state.map_matrix[nidx];
    state.map_matrix[idx] = Tile::Empty as u8;
    state.map_matrix[nidx] = Tile::Player as u8;
    state.player_idx = nidx;
    let _ = consume_tile_effect(state, target_tile);
    if target_tile == Tile::Empty as u8 {
        state.feedback = "Moved successfully.".to_string();
    }
    true
}

/// Fire into an adjacent tile, consuming AP and resolving any tile effect.
pub fn fire_at_direction(state: &mut GameState, dx: isize, dy: isize) -> bool {
    let idx = state.player_idx;
    if state.stats.ap < 2 {
        state.feedback = "Not enough AP to fire (needs 2).".to_string();
        return false;
    }
    state.stats.ap = state.stats.ap.saturating_sub(2);
    let mut cx = (idx % state.width) as isize;
    let mut cy = (idx / state.width) as isize;

    loop {
        cx += dx;
        cy += dy;
        if cx < 0 || cy < 0 || cx >= state.width as isize || cy >= state.height as isize {
            break;
        }
        let nidx = (cy as usize) * state.width + (cx as usize);
        let target = state.map_matrix[nidx];
        if target == Tile::Wall as u8 {
            break;
        }
        if target == Tile::Enemy as u8 {
            state.map_matrix[nidx] = Tile::Wreck as u8;
            state.feedback = "Target destroyed!".to_string();
            return true;
        } else if target != Tile::Empty as u8 {
            state.map_matrix[nidx] = Tile::Empty as u8;
            state.feedback = "Obstacle cleared.".to_string();
            return true;
        }
    }
    state.feedback = "Missed.".to_string();
    false
}

fn consume_tile_effect(state: &mut GameState, tile: u8) -> bool {
    match tile {
        x if x == Tile::Health as u8 => {
            state.stats.health = state.stats.health.saturating_add(20).min(100);
            state.feedback = "Picked up Health: +20 HP".to_string();
            true
        }
        x if x == Tile::Smoke as u8 => {
            state.stats.active_item = 1;
            state.stats.item_charges = 1;
            state.feedback = "Acquired Smoke charge.".to_string();
            true
        }
        x if x == Tile::Resource as u8 => {
            state.stats.ap = state.stats.ap.saturating_add(3).min(12);
            state.feedback = "Acquired Resource: +3 AP".to_string();
            true
        }
        x if x == Tile::Mine as u8 => {
            state.stats.health = state.stats.health.saturating_sub(15);
            state.feedback = "Hit a Mine! -15 HP".to_string();
            true
        }
        x if x == Tile::Wreck as u8 => {
            state.stats.armor = state.stats.armor.saturating_sub(10);
            state.feedback = "Scraped wreckage: -10 Armor".to_string();
            true
        }
        _ => false,
    }
}

/// Spawn occasional drops into empty tiles; simple probability per call.
/// Spawn new pickups or hazards into empty tiles using a seeded RNG.
/// Toggle overdrive (supercharging) state.
pub fn toggle_overdrive(state: &mut GameState) -> bool {
    state.stats.is_supercharging = !state.stats.is_supercharging;
    if state.stats.is_supercharging {
        state.feedback = "Overdrive engaged!".to_string();
    } else {
        state.feedback = "Overdrive disengaged.".to_string();
    }
    false
}

pub fn spawn_drops(state: &mut GameState, rng_seed: u64) {
    use rand::Rng;
    use rand::SeedableRng;
    use rand::rngs::StdRng;
    let mut rng = StdRng::seed_from_u64(rng_seed);
    let size = state.width * state.height;
    for _ in 0..3 {
        let idx = rng.gen_range(0..size);
        if state.map_matrix[idx] == Tile::Empty as u8 {
            let pick = rng.gen_range(0..100);
            if pick < 8 {
                state.map_matrix[idx] = Tile::Health as u8;
            } else if pick < 15 {
                state.map_matrix[idx] = Tile::Mine as u8;
            } else if pick < 28 {
                state.map_matrix[idx] = Tile::Resource as u8;
            } else if pick < 38 {
                state.map_matrix[idx] = Tile::Smoke as u8;
            } else if pick < 43 {
                state.map_matrix[idx] = Tile::Wreck as u8;
            }
        }
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn deterministic_map_same_seed() {
        let (a, _) = generate_map(42, 8, 8);
        let (b, _) = generate_map(42, 8, 8);
        assert_eq!(a, b);
    }

    #[test]
    fn different_seed_differs() {
        let (a, _) = generate_map(1, 8, 8);
        let (b, _) = generate_map(2, 8, 8);
        assert_ne!(a, b);
    }

    #[test]
    fn map_size_matches() {
        let (a, _) = generate_map(7, 10, 6);
        assert_eq!(a.len(), 60);
    }

    #[test]
    fn apply_health_pickup_works() {
        let mut gs = GameState {
            phase: AppPhase::Playing,
            stats: PlayerStats {
                health: 50,
                armor: 10,
                ap: 6,
                is_supercharging: false,
                has_shield: false,
                active_item: 0,
                item_charges: 0,
            },
            map_matrix: vec![Tile::Health as u8],
            width: 1,
            height: 1,
            seed: 1,
            player_idx: 0,
            feedback: "".to_string(),
        };
        let applied = consume_tile_effect(&mut gs, Tile::Health as u8);
        gs.map_matrix[0] = Tile::Empty as u8;
        assert!(applied);
        assert_eq!(gs.stats.health, 70);
        assert_eq!(gs.map_matrix[0], Tile::Empty as u8);
    }

    #[test]
    fn player_movement_and_collision() {
        let mut gs = GameState {
            phase: AppPhase::Playing,
            stats: PlayerStats {
                health: 100,
                armor: 0,
                ap: 3,
                is_supercharging: false,
                has_shield: false,
                active_item: 0,
                item_charges: 0,
            },
            map_matrix: vec![Tile::Player as u8, Tile::Wall as u8, Tile::Empty as u8],
            width: 3,
            height: 1,
            seed: 1,
            player_idx: 0,
            feedback: "".to_string(),
        };
        // try to move right into wall (should fail)
        assert!(!move_player(&mut gs, 1, 0));

        assert_eq!(gs.stats.ap, 3);
        // move to empty (index 2) by shifting wall away
        gs.map_matrix[1] = Tile::Empty as u8;
        assert!(move_player(&mut gs, 1, 0));
        assert_eq!(gs.stats.ap, 2);
    }
}

#[cfg(test)]
mod fire_tests {
    use super::*;

    #[test]
    fn test_fire_at_direction_raycast() {
        let mut gs = GameState {
            phase: AppPhase::Playing,
            stats: PlayerStats {
                health: 100,
                armor: 0,
                ap: 4,
                is_supercharging: false,
                has_shield: false,
                active_item: 0,
                item_charges: 0,
            },
            map_matrix: vec![
                Tile::Player as u8,
                Tile::Empty as u8,
                Tile::Empty as u8,
                Tile::Enemy as u8,
            ],
            width: 4,
            height: 1,
            seed: 1,
            player_idx: 0,
            feedback: "".to_string(),
        };

        // Fire right
        let hit = fire_at_direction(&mut gs, 1, 0);
        assert!(hit);
        assert_eq!(gs.stats.ap, 2);
        assert_eq!(gs.map_matrix[3], Tile::Wreck as u8);

        // Fire again, should hit the wreck and turn it into empty
        let hit2 = fire_at_direction(&mut gs, 1, 0);
        assert!(hit2);
        assert_eq!(gs.stats.ap, 0);
        assert_eq!(gs.map_matrix[3], Tile::Empty as u8);

        // Fire again, not enough AP
        let hit3 = fire_at_direction(&mut gs, 1, 0);
        assert!(!hit3);
    }
}
