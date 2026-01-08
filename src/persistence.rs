use crate::game_state::GameState;
use macroquad_toolkit::persistence::{save_json, load_json, get_app_data_path};

const SAVE_FILE: &str = "dungeon_core_save.json";

/// Save game state to JSON file
pub fn save_game(state: &GameState) -> Result<(), String> {
    // Attempt to use app data path, fallback to local file
    let path = get_app_data_path("dungeon_core", SAVE_FILE)
        .unwrap_or_else(|| std::path::PathBuf::from(SAVE_FILE));
        
    save_json(path, state)
}

/// Load game state from JSON file
pub fn load_game() -> Result<GameState, String> {
    // Attempt to use app data path, fallback to local file
    let path = get_app_data_path("dungeon_core", SAVE_FILE)
        .unwrap_or_else(|| std::path::PathBuf::from(SAVE_FILE));

    load_json(path)
}
