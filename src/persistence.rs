use crate::game_state::GameState;
use macroquad_toolkit::persistence::{json_key_exists, load_json_key, save_json_key};

const SAVE_FILE: &str = "dungeon_core_save.json";
const GAME_NAME: &str = "dungeon_core";

/// Save game state to JSON file
pub fn save_game(state: &GameState) -> Result<(), String> {
    save_json_key(GAME_NAME, SAVE_FILE, state)
}

/// Load game state from JSON file
pub fn load_game() -> Result<GameState, String> {
    load_json_key(GAME_NAME, SAVE_FILE)
}

/// Check whether a saved dungeon exists in the active platform storage.
pub fn save_exists() -> bool {
    json_key_exists(GAME_NAME, SAVE_FILE)
}
