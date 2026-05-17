use serde::Deserialize;

/// Adventurer class from JSON
#[derive(Clone, Debug, Deserialize)]
pub struct AdventurerClass {
    pub name: String,
    pub hp: i32,
    pub attack: i32,
    pub defense: i32,
}

/// Dialogue quotes
#[derive(Debug, Deserialize)]
pub struct QuotesData {
    pub victory: Vec<String>,
    pub entry: Vec<String>,
    pub exit: Vec<String>,
}

#[derive(Debug, Deserialize)]
struct AdventurersData {
    classes: Vec<AdventurerClass>,
    names: Vec<String>,
    quotes: QuotesData,
}

// Embed JSON at compile time for WASM compatibility
const ADVENTURERS_JSON: &str = include_str!("../../assets/adventurers.json");

fn load_data() -> AdventurersData {
    serde_json::from_str(ADVENTURERS_JSON).expect("Failed to parse adventurers.json")
}

/// Get all adventurer classes
pub fn get_adventurer_classes() -> Vec<AdventurerClass> {
    load_data().classes
}

/// Get adventurer class by name
pub fn get_adventurer_class(name: &str) -> Option<AdventurerClass> {
    get_adventurer_classes()
        .into_iter()
        .find(|c| c.name == name)
}

/// Get all adventurer names
pub fn get_adventurer_names() -> Vec<String> {
    load_data().names
}

/// Get victory quotes
pub fn get_victory_quotes() -> Vec<String> {
    load_data().quotes.victory
}

/// Get entry quotes
pub fn get_entry_quotes() -> Vec<String> {
    load_data().quotes.entry
}

/// Get exit quotes
pub fn get_exit_quotes() -> Vec<String> {
    load_data().quotes.exit
}
