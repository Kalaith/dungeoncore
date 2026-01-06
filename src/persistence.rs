use crate::game_state::GameState;
use std::fs;

const SAVE_FILE: &str = "dungeon_core_save.json";

/// Save game state to JSON file
pub fn save_game(state: &GameState) -> Result<(), String> {
    let json = serde_json::to_string_pretty(state).map_err(|e| e.to_string())?;
    fs::write(SAVE_FILE, json).map_err(|e| e.to_string())
}

/// Load game state from JSON file
pub fn load_game() -> Result<GameState, String> {
    let data = fs::read_to_string(SAVE_FILE).map_err(|e| e.to_string())?;
    serde_json::from_str(&data).map_err(|e| e.to_string())
}
