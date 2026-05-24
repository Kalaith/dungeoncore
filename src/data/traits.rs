use serde::{Deserialize, Serialize};
use serde_json::Value;

#[derive(Clone, Debug, Serialize, Deserialize)]
pub struct MonsterTrait {
    pub id: String,
    pub name: String,
    pub description: String,
    pub trait_type: String,  // "Passive", "Active"
    pub target_type: String, // "Self", "Enemy", "EnemyParty", "Ally"
    pub applies_to: String,  // "Hourly", "OnDefense", "OnAttack", "OnCombatStart"
    #[serde(default)]
    pub effect_type: String, // "HealPercent", "DamageFlat", "DamageReductionMult", "AttackBonus"
    #[serde(default)]
    pub scaling_type: String, // "None", "PerAlly", "PerEnemy"
    pub mana_cost: i32,
    pub cooldown: i32,
    pub value: f32,
}

// Embed JSON for WASM
const TRAITS_JSON: &str = include_str!("../../assets/traits.json");

/// Load all traits
pub fn get_all_traits() -> Vec<MonsterTrait> {
    let Ok(data) = serde_json::from_str::<Value>(TRAITS_JSON) else {
        return Vec::new();
    };

    data.get("traits")
        .and_then(Value::as_array)
        .map(|traits| traits.iter().filter_map(parse_trait).collect())
        .unwrap_or_default()
}

/// Get a specific trait by ID
pub fn get_trait(id: &str) -> Option<MonsterTrait> {
    get_all_traits().into_iter().find(|t| t.id == id)
}

fn parse_trait(value: &Value) -> Option<MonsterTrait> {
    let id = value.get("id")?.as_str()?.to_string();

    Some(MonsterTrait {
        id,
        name: string_field(value, "name", "Unknown Trait"),
        description: string_field(value, "description", ""),
        trait_type: string_field(value, "trait_type", ""),
        target_type: string_field(value, "target_type", "Self"),
        applies_to: string_field(value, "applies_to", ""),
        effect_type: string_field(value, "effect_type", "None"),
        scaling_type: string_field(value, "scaling_type", "None"),
        mana_cost: int_field(value, "mana_cost"),
        cooldown: int_field(value, "cooldown"),
        value: float_field(value, "value"),
    })
}

fn string_field(value: &Value, key: &str, default: &str) -> String {
    value
        .get(key)
        .and_then(Value::as_str)
        .unwrap_or(default)
        .to_string()
}

fn int_field(value: &Value, key: &str) -> i32 {
    value.get(key).and_then(Value::as_i64).unwrap_or_default() as i32
}

fn float_field(value: &Value, key: &str) -> f32 {
    value.get(key).and_then(Value::as_f64).unwrap_or_default() as f32
}
