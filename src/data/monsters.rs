use serde::{Deserialize, Serialize};

/// Monster template from JSON
#[derive(Clone, Debug, Serialize, Deserialize)]
pub struct MonsterTemplate {
    pub name: String,
    pub base_cost: i32,
    pub hp: i32,
    pub attack: i32,
    pub defense: i32,
    pub species: String,
    pub tier: i32,
    pub emoji: String,
    #[serde(default)]
    pub element: Option<String>,
    #[serde(default)]
    pub description: String,
    #[serde(default)]
    pub traits: Vec<String>,
    /// Souls required to summon, on top of mana (Demons)
    #[serde(default)]
    pub souls_cost: i32,
}

/// Species data from JSON
#[derive(Clone, Debug, Serialize, Deserialize)]
pub struct SpeciesData {
    pub name: String,
    #[serde(default)]
    pub display_name: Option<String>,
    #[serde(default)]
    pub starter: bool,
    pub unlock_cost: i32,
    pub description: String,
}

#[derive(Debug, Deserialize)]
struct MonstersData {
    monsters: Vec<MonsterTemplate>,
    species: Vec<SpeciesData>,
}

// Embed JSON at compile time for WASM compatibility
const MONSTERS_JSON: &str = include_str!("../../assets/monsters.json");

/// Load all monster templates from embedded JSON
pub fn get_monster_templates() -> Vec<MonsterTemplate> {
    let data: MonstersData =
        serde_json::from_str(MONSTERS_JSON).expect("Failed to parse monsters.json");
    data.monsters
}

/// Find a monster template by name
pub fn get_monster_template(name: &str) -> Option<MonsterTemplate> {
    get_monster_templates().into_iter().find(|t| t.name == name)
}

/// Get all species data
pub fn get_all_species() -> Vec<SpeciesData> {
    let data: MonstersData =
        serde_json::from_str(MONSTERS_JSON).expect("Failed to parse monsters.json");
    data.species
}

/// Get one species record by internal ID.
pub fn get_species(species_name: &str) -> Option<SpeciesData> {
    get_all_species()
        .into_iter()
        .find(|s| s.name == species_name)
}

/// Get species unlock cost
pub fn get_species_unlock_cost(species_name: &str) -> Option<i32> {
    get_species(species_name).map(|s| s.unlock_cost)
}

/// Human-facing species name; keeps save/internal IDs stable.
pub fn get_species_display_name(species_name: &str) -> String {
    get_all_species()
        .into_iter()
        .find(|s| s.name == species_name)
        .and_then(|s| s.display_name)
        .unwrap_or_else(|| species_name.to_string())
}

/// Starter roster for a race/species. Higher tiers remain progression unlocks.
pub fn get_starter_monsters_for_species(species_name: &str) -> Vec<MonsterTemplate> {
    get_monster_templates()
        .into_iter()
        .filter(|template| template.species == species_name && template.tier == 1)
        .collect()
}

/// Get all unique species names
pub fn get_species_names() -> Vec<String> {
    get_all_species().into_iter().map(|s| s.name).collect()
}
