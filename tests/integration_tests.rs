use grid_crawler_wsl::state::{
    AppPhase, GameState, PlayerStats, Tile, fire_at_direction, move_player,
};

#[test]
fn test_player_sequence_move_pickup_fire() {
    let mut gs = GameState {
        phase: AppPhase::Playing,
        stats: PlayerStats {
            health: 50,
            armor: 10,
            ap: 4,
            is_supercharging: false,
            has_shield: false,
            active_item: 0,
            item_charges: 0,
        },
        map_matrix: vec![
            Tile::Player as u8,
            Tile::Health as u8,
            Tile::Empty as u8,
            Tile::Enemy as u8,
        ],
        width: 4,
        height: 1,
        seed: 1,
        player_idx: 0,
        feedback: "".to_string(),
    };

    // Step 1: Move right onto Health
    assert!(move_player(&mut gs, 1, 0));
    assert_eq!(gs.stats.ap, 3); // Cost 1 AP
    assert_eq!(gs.stats.health, 70); // Health increased
    assert_eq!(gs.player_idx, 1);
    assert_eq!(gs.map_matrix[0], Tile::Empty as u8); // Old position empty
    assert_eq!(gs.map_matrix[1], Tile::Player as u8); // New position player

    // Step 2: Fire right
    assert!(fire_at_direction(&mut gs, 1, 0));
    assert_eq!(gs.stats.ap, 1); // Cost 2 AP
    assert_eq!(gs.map_matrix[3], Tile::Wreck as u8); // Enemy turns to wreck

    // Step 3: Try to fire again (not enough AP)
    assert!(!fire_at_direction(&mut gs, 1, 0));
    assert_eq!(gs.stats.ap, 1); // AP unchanged
}

#[test]
fn test_player_sequence_move_obstacle_mine() {
    let mut gs = GameState {
        phase: AppPhase::Playing,
        stats: PlayerStats {
            health: 100,
            armor: 50,
            ap: 4,
            is_supercharging: false,
            has_shield: false,
            active_item: 0,
            item_charges: 0,
        },
        map_matrix: vec![
            Tile::Player as u8,
            Tile::Wall as u8,
            Tile::Empty as u8,
            Tile::Mine as u8,
        ],
        width: 2,
        height: 2,
        seed: 1,
        player_idx: 0,
        feedback: "".to_string(),
    };

    // Try moving right into wall
    assert!(!move_player(&mut gs, 1, 0));
    assert_eq!(gs.stats.ap, 4); // AP unchanged
    assert_eq!(gs.player_idx, 0); // Position unchanged

    // Move down into empty space
    assert!(move_player(&mut gs, 0, 1));
    assert_eq!(gs.stats.ap, 3); // Cost 1 AP
    assert_eq!(gs.player_idx, 2);

    // Move right into mine
    assert!(move_player(&mut gs, 1, 0));
    assert_eq!(gs.stats.ap, 2); // Cost 1 AP
    assert_eq!(gs.player_idx, 3);
    assert_eq!(gs.stats.health, 85); // Hit mine
}
