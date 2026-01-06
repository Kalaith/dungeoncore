use serde::{Deserialize, Serialize};

/// Equipment types
#[derive(Clone, Debug, PartialEq, Serialize, Deserialize)]
pub enum EquipmentType {
    Weapon,
    Armor,
    Accessory,
}

impl EquipmentType {
    pub fn from_str(s: &str) -> Self {
        match s {
            "weapon" => EquipmentType::Weapon,
            "armor" => EquipmentType::Armor,
            "accessory" => EquipmentType::Accessory,
            _ => panic!("Unknown equipment type: {}", s),
        }
    }
}

/// JSON-loadable equipment template
#[derive(Clone, Debug, Serialize, Deserialize)]
pub struct EquipmentTemplate {
    pub id: u32,
    pub name: String,
    #[serde(rename = "type")]
    pub equipment_type: String,
    pub level: u32,
    pub attack: i32,
    pub defense: i32,
    pub magic: i32,
    pub cost: i32,
    pub description: String,
    pub date: String,
}

#[derive(Debug, Deserialize)]
struct EquipmentData {
    equipment: Vec<EquipmentTemplate>,
}

// Embed JSON at compile time for WASM compatibility
const EQUIPMENT_JSON: &str = include_str!("../../assets/equipment.json");

/// Load all equipment templates from embedded JSON
pub fn get_all_equipment() -> Vec<EquipmentTemplate> {
    let data: EquipmentData = serde_json::from_str(EQUIPMENT_JSON)
        .expect("Failed to parse equipment.json");
    data.equipment
}

/// Find equipment template by ID
pub fn get_equipment_template(id: u32) -> Option<EquipmentTemplate> {
    get_all_equipment().into_iter().find(|e| e.id == id)
}

/// Find equipment template by name
pub fn get_equipment_template_by_name(name: &str) -> Option<EquipmentTemplate> {
    get_all_equipment().into_iter().find(|e| e.name == name)
}

/// Get equipment of a specific type
pub fn get_equipment_by_type(equipment_type: &str) -> Vec<EquipmentTemplate> {
    get_all_equipment()
        .into_iter()
        .filter(|e| e.equipment_type == equipment_type)
        .collect()
}

/// Get equipment by level
pub fn get_equipment_by_level(level: u32) -> Vec<EquipmentTemplate> {
    get_all_equipment()
        .into_iter()
        .filter(|e| e.level == level)
        .collect()
}

/// Get equipment within a cost range
pub fn get_equipment_by_cost_range(min_cost: i32, max_cost: i32) -> Vec<EquipmentTemplate> {
    get_all_equipment()
        .into_iter()
        .filter(|e| e.cost >= min_cost && e.cost <= max_cost)
        .collect()
}