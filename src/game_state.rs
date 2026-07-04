use serde::{Deserialize, Serialize};

/// Combat stats for monsters and adventurers
#[derive(Clone, Copy, Debug, Default, Serialize, Deserialize)]
pub struct Stats {
    pub hp: i32,
    pub attack: i32,
    pub defense: i32,
}

/// Room type enumeration
#[derive(Clone, Debug, PartialEq, Serialize, Deserialize)]
pub enum RoomType {
    Entrance,
    Normal,
    Boss,
    Core,
}

/// Room upgrade type
#[derive(Clone, Debug, PartialEq, Serialize, Deserialize)]
pub enum RoomUpgradeType {
    Trap,
    Treasure,
    Reinforcement,
    Evolution,
}

/// Room upgrade applied to a room
#[derive(Clone, Debug, Serialize, Deserialize)]
pub struct RoomUpgrade {
    pub upgrade_type: RoomUpgradeType,
    pub name: String,
    pub effect: String,
    pub multiplier: f32,
}

/// Active trait instance on a monster
#[derive(Clone, Debug, Serialize, Deserialize)]
pub struct ActiveTrait {
    pub id: String,
    pub name: String,
    pub cooldown_timer: i32,
}

/// Monster instance in a room
#[derive(Clone, Debug, Serialize, Deserialize)]
pub struct Monster {
    pub id: u64,
    pub type_name: String,
    pub hp: i32,
    pub max_hp: i32,
    pub alive: bool,
    pub is_boss: bool,
    pub scaled_stats: Stats,
    #[serde(default)]
    pub active_traits: Vec<ActiveTrait>,
    #[serde(default)]
    pub experience: i32,
}

/// Room in a dungeon floor
#[derive(Clone, Debug, Serialize, Deserialize)]
pub struct Room {
    pub id: u64,
    pub room_type: RoomType,
    pub position: usize,
    pub floor_number: i32,
    pub monsters: Vec<Monster>,
    pub upgrade: Option<RoomUpgrade>,
    pub explored: bool,
    pub loot: i32,
}

impl Room {
    pub fn new(id: u64, room_type: RoomType, position: usize, floor_number: i32) -> Self {
        Self {
            id,
            room_type,
            position,
            floor_number,
            monsters: Vec::new(),
            upgrade: None,
            explored: false,
            loot: 0,
        }
    }

    /// Get the trap damage multiplier (from trap upgrades)
    pub fn trap_multiplier(&self) -> f32 {
        match &self.upgrade {
            Some(u) if u.upgrade_type == RoomUpgradeType::Trap => u.multiplier,
            _ => 1.0,
        }
    }

    /// Get the treasure/loot multiplier
    pub fn treasure_multiplier(&self) -> f32 {
        match &self.upgrade {
            Some(u) if u.upgrade_type == RoomUpgradeType::Treasure => u.multiplier,
            _ => 1.0,
        }
    }

    /// Get monster stat boost from reinforcement
    pub fn reinforcement_multiplier(&self) -> f32 {
        match &self.upgrade {
            Some(u) if u.upgrade_type == RoomUpgradeType::Reinforcement => u.multiplier,
            _ => 1.0,
        }
    }

    /// Get XP multiplier from evolution upgrade
    pub fn evolution_multiplier(&self) -> f32 {
        match &self.upgrade {
            Some(u) if u.upgrade_type == RoomUpgradeType::Evolution => u.multiplier,
            _ => 1.0,
        }
    }
}

/// Floor in the dungeon
#[derive(Clone, Debug, Serialize, Deserialize)]
pub struct Floor {
    pub id: u64,
    pub number: i32,
    pub rooms: Vec<Room>,
    pub is_deepest: bool,
}

impl Floor {
    pub fn new(id: u64, number: i32, is_deepest: bool) -> Self {
        Self {
            id,
            number,
            rooms: Vec::new(),
            is_deepest,
        }
    }
}

/// Adventurer equipment
#[derive(Clone, Debug, Serialize, Deserialize)]
pub struct Equipment {
    pub weapon: String,
    pub armor: String,
    pub accessory: String,
}

impl Default for Equipment {
    fn default() -> Self {
        Self {
            weapon: "Rusty Sword".into(),
            armor: "Cloth Robe".into(),
            accessory: "Worn Ring".into(),
        }
    }
}

/// Individual adventurer in a party
#[derive(Clone, Debug, Serialize, Deserialize)]
pub struct Adventurer {
    pub id: u64,
    pub name: String,
    pub class_name: String,
    pub level: i32,
    pub hp: i32,
    pub max_hp: i32,
    pub alive: bool,
    pub experience: i32,
    pub gold: i32,
    pub equipment: Equipment,
    pub conditions: Vec<String>,
    pub scaled_stats: Stats,
}

/// Party of adventurers exploring the dungeon
#[derive(Clone, Debug, Serialize, Deserialize)]
pub struct AdventurerParty {
    pub id: u64,
    pub members: Vec<Adventurer>,
    pub current_floor: i32,
    pub current_room: usize,
    pub retreating: bool,
    pub casualties: i32,
    pub loot: i32,
    pub entry_time: i32,
    pub target_floor: i32,
}

/// Dungeon operational status
#[derive(Clone, Debug, PartialEq, Serialize, Deserialize)]
pub enum DungeonStatus {
    Open,
    Closing,
    Closed,
    Maintenance,
}

/// Kind of transient visual effect surfaced over a room
#[derive(Clone, Copy, Debug, PartialEq)]
pub enum EffectKind {
    Damage,
    Ability,
    MonsterDown,
    AdventurerDown,
    Loot,
}

/// A short-lived floating effect anchored to a room (not persisted)
#[derive(Clone, Debug)]
pub struct RoomEffect {
    pub floor: i32,
    pub room: usize,
    pub text: String,
    pub kind: EffectKind,
    pub ttl: f32,
    pub max_ttl: f32,
}

/// Log entry type
#[derive(Clone, Debug, Serialize, Deserialize)]
pub struct LogEntry {
    pub message: String,
    pub log_type: String, // "system", "combat", "adventure", "building"
    pub timestamp: u64,
}

impl LogEntry {
    pub fn new(message: impl Into<String>, log_type: impl Into<String>) -> Self {
        Self {
            message: message.into(),
            log_type: log_type.into(),
            timestamp: 0,
        }
    }

    pub fn system(message: impl Into<String>) -> Self {
        Self::new(message, "system")
    }

    pub fn combat(message: impl Into<String>) -> Self {
        Self::new(message, "combat")
    }

    pub fn adventure(message: impl Into<String>) -> Self {
        Self::new(message, "adventure")
    }

    pub fn building(message: impl Into<String>) -> Self {
        Self::new(message, "building")
    }
}

/// Main game state - mirrors GameState from types/game.ts
#[derive(Clone, Debug, Serialize, Deserialize)]
pub struct GameState {
    // Resources
    pub mana: i32,
    pub max_mana: i32,
    pub mana_regen: f32,
    pub gold: i32,
    pub souls: i32,

    // Time
    pub day: i32,
    pub hour: i32,
    pub speed: i32,

    // Dungeon
    pub status: DungeonStatus,
    pub floors: Vec<Floor>,
    pub total_floors: i32,
    pub deep_core_bonus: f32,

    // Adventurers
    pub adventurer_parties: Vec<AdventurerParty>,
    pub next_party_spawn: i32,

    // Reputation / threat
    #[serde(default)]
    pub total_deaths: i32,
    #[serde(default)]
    pub threat_warned: i32,
    #[serde(default)]
    pub raids_completed: i32,

    // Onboarding tutorial (only enabled for fresh games)
    #[serde(default)]
    pub tutorial_active: bool,
    #[serde(default)]
    pub tutorial_step: i32,

    // Monster progression
    pub unlocked_species: Vec<String>,
    pub unlocked_monsters: Vec<String>,

    // UI state (not persisted)
    #[serde(skip)]
    pub selected_room: Option<(i32, usize)>,
    #[serde(skip)]
    pub selected_monster: Option<String>,
    #[serde(skip)]
    pub effects: Vec<RoomEffect>,

    // Log
    pub log: Vec<LogEntry>,
}

impl Default for GameState {
    fn default() -> Self {
        Self::new()
    }
}

impl GameState {
    pub fn new() -> Self {
        // Create initial floor with entrance and core rooms
        let mut floor1 = Floor::new(1, 1, true);
        floor1.rooms.push(Room::new(1, RoomType::Entrance, 0, 1));
        floor1.rooms.push(Room::new(2, RoomType::Core, 1, 1));

        Self {
            mana: 100,
            max_mana: 200,
            mana_regen: 1.0,
            gold: 50,
            souls: 0,
            day: 1,
            hour: 6,
            speed: 1,
            status: DungeonStatus::Closed,
            floors: vec![floor1],
            total_floors: 1,
            deep_core_bonus: 0.1,
            adventurer_parties: Vec::new(),
            next_party_spawn: 8,
            total_deaths: 0,
            threat_warned: 0,
            raids_completed: 0,
            tutorial_active: true,
            tutorial_step: 0,
            unlocked_species: vec![],
            unlocked_monsters: vec![],
            selected_room: None,
            selected_monster: None,
            effects: Vec::new(),
            log: vec![LogEntry::system(
                "Welcome to Dungeon Core! Choose a starter race to awaken your first defenders.",
            )],
        }
    }

    /// Add a log entry, keeping max entries
    pub fn add_log(&mut self, entry: LogEntry) {
        self.log.push(entry);
        if self.log.len() > crate::data::MAX_LOG_ENTRIES {
            self.log.remove(0);
        }
    }

    /// Spawn a short-lived floating effect over a room
    pub fn push_effect(
        &mut self,
        floor: i32,
        room: usize,
        text: impl Into<String>,
        kind: EffectKind,
    ) {
        const EFFECT_TTL: f32 = 1.6;
        self.effects.push(RoomEffect {
            floor,
            room,
            text: text.into(),
            kind,
            ttl: EFFECT_TTL,
            max_ttl: EFFECT_TTL,
        });
        if self.effects.len() > 48 {
            self.effects.remove(0);
        }
    }

    /// Age floating effects and drop expired ones
    pub fn decay_effects(&mut self, dt: f32) {
        for effect in &mut self.effects {
            effect.ttl -= dt;
        }
        self.effects.retain(|effect| effect.ttl > 0.0);
    }

    /// Current threat tier (0-4) derived from accumulated adventurer deaths
    pub fn threat_tier(&self) -> i32 {
        match self.total_deaths {
            d if d >= 100 => 4,
            d if d >= 50 => 3,
            d if d >= 25 => 2,
            d if d >= 10 => 1,
            _ => 0,
        }
    }

    /// Get the deepest floor
    pub fn deepest_floor(&self) -> Option<&Floor> {
        self.floors.iter().find(|f| f.is_deepest)
    }

    /// Get mutable reference to the deepest floor
    pub fn deepest_floor_mut(&mut self) -> Option<&mut Floor> {
        self.floors.iter_mut().find(|f| f.is_deepest)
    }

    /// Count total rooms (excluding entrance and core)
    pub fn total_room_count(&self) -> i32 {
        self.floors
            .iter()
            .flat_map(|f| &f.rooms)
            .filter(|r| r.room_type != RoomType::Core && r.room_type != RoomType::Entrance)
            .count() as i32
    }
}
