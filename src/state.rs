use std::sync::{Arc, Mutex};

#[derive(PartialEq)]
pub enum AppPhase {
    StartScreen,
    Playing,
    GameOver,
    Scoreboard,
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
}

pub type SharedState = Arc<Mutex<GameState>>;

pub fn initialize_state() -> SharedState {
    let mut default_map = vec![0x00; 64]; 
    
    // Injecting a mocked tactical layout for UI verification
    default_map[3 * 8 + 3] = 0x01; // Wall
    default_map[3 * 8 + 4] = 0x01; // Wall
    default_map[3 * 8 + 5] = 0x01; // Wall
    default_map[4 * 8 + 2] = 0x01; // Wall
    default_map[7 * 8 + 3] = 0x0A; // Player 1 (WSL)
    default_map[1 * 8 + 4] = 0x0B; // Player 2 (Mobile)
    default_map[2 * 8 + 6] = 0x10; // Item Pickup

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
        map_matrix: default_map,
    };

    Arc::new(Mutex::new(initial_state))
}