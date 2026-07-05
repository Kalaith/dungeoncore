use crate::data::upgrades::{get_upgrade_template, UpgradeTemplate};
use crate::game_state::{GameState, LogEntry, RoomType, RoomUpgradeType};

/// Apply an upgrade to a room
pub fn apply_upgrade(
    state: &mut GameState,
    floor_num: i32,
    room_pos: usize,
    upgrade_name: &str,
) -> Result<(), String> {
    // Cannot upgrade while adventurers are in dungeon
    if !state.adventurer_parties.is_empty() {
        return Err("Cannot upgrade rooms while adventurers are in the dungeon!".into());
    }

    // Find upgrade template
    let template = get_upgrade_template(upgrade_name)
        .ok_or_else(|| format!("Unknown upgrade: {}", upgrade_name))?;

    // Find floor and room
    let floor = state
        .floors
        .iter_mut()
        .find(|f| f.number == floor_num)
        .ok_or("Floor not found")?;

    let room = floor
        .rooms
        .iter_mut()
        .find(|r| r.position == room_pos)
        .ok_or("Room not found")?;

    // Cannot upgrade entrance or core rooms
    if room.room_type == RoomType::Entrance || room.room_type == RoomType::Core {
        return Err("Cannot upgrade entrance or core rooms!".into());
    }

    // One upgrade per type: a room can hold a trap AND a treasure, but not
    // two traps.
    let upgrade_type = crate::data::upgrades::parse_upgrade_type(&template.upgrade_type);
    if room.has_upgrade_type(upgrade_type) {
        return Err(format!(
            "Room already has a {} upgrade! Remove it first.",
            template.upgrade_type
        ));
    }

    // Upgrades are conjured by the core: they cost mana (and sometimes souls).
    if state.mana < template.mana_cost {
        return Err(format!(
            "Not enough mana! Need {} mana.",
            template.mana_cost
        ));
    }
    if state.souls < template.souls_cost {
        return Err(format!(
            "Not enough souls! Need {} souls.",
            template.souls_cost
        ));
    }

    // Deduct costs
    state.mana -= template.mana_cost;
    state.souls -= template.souls_cost;

    // Apply upgrade
    room.upgrades.push(template.to_room_upgrade());

    state.add_log(LogEntry::building(format!(
        "Applied {} to floor {}, room {} for {} mana{}",
        upgrade_name,
        floor_num,
        room_pos,
        template.mana_cost,
        if template.souls_cost > 0 {
            format!(" and {} souls", template.souls_cost)
        } else {
            String::new()
        }
    )));

    Ok(())
}

/// Remove the upgrade of a given type from a room (no refund)
pub fn remove_upgrade(
    state: &mut GameState,
    floor_num: i32,
    room_pos: usize,
    upgrade_type: RoomUpgradeType,
) -> Result<(), String> {
    // Cannot modify while adventurers are in dungeon
    if !state.adventurer_parties.is_empty() {
        return Err("Cannot modify rooms while adventurers are in the dungeon!".into());
    }

    // Find floor and room
    let floor = state
        .floors
        .iter_mut()
        .find(|f| f.number == floor_num)
        .ok_or("Floor not found")?;

    let room = floor
        .rooms
        .iter_mut()
        .find(|r| r.position == room_pos)
        .ok_or("Room not found")?;

    let idx = room
        .upgrades
        .iter()
        .position(|u| u.upgrade_type == upgrade_type)
        .ok_or("Room has no upgrade of that type to remove.")?;
    let upgrade_name = room.upgrades.remove(idx).name;

    state.add_log(LogEntry::building(format!(
        "Removed {} from floor {}, room {}",
        upgrade_name, floor_num, room_pos
    )));

    Ok(())
}

/// Get available upgrades for a room
pub fn get_available_upgrades(
    state: &GameState,
    floor_num: i32,
    room_pos: usize,
) -> Result<Vec<UpgradeTemplate>, String> {
    // Find floor and room
    let floor = state
        .floors
        .iter()
        .find(|f| f.number == floor_num)
        .ok_or("Floor not found")?;

    let room = floor
        .rooms
        .iter()
        .find(|r| r.position == room_pos)
        .ok_or("Room not found")?;

    // Cannot upgrade entrance or core
    if room.room_type == RoomType::Entrance || room.room_type == RoomType::Core {
        return Ok(Vec::new());
    }

    // Offer upgrades of types the room does not hold yet.
    Ok(crate::data::get_all_upgrades()
        .into_iter()
        .filter(|t| {
            !room.has_upgrade_type(crate::data::upgrades::parse_upgrade_type(&t.upgrade_type))
        })
        .collect())
}

/// Check if an upgrade can be afforded
pub fn can_afford_upgrade(state: &GameState, template: &UpgradeTemplate) -> bool {
    state.mana >= template.mana_cost && state.souls >= template.souls_cost
}

/// Get upgrade type icon/emoji
pub fn upgrade_type_icon(upgrade_type: &RoomUpgradeType) -> &'static str {
    match upgrade_type {
        RoomUpgradeType::Trap => "⚡",
        RoomUpgradeType::Treasure => "💰",
        RoomUpgradeType::Reinforcement => "🛡️",
        RoomUpgradeType::Evolution => "🧬",
        RoomUpgradeType::Attunement => "🔮",
    }
}
