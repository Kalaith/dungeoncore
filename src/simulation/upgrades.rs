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

    // Check if room already has an upgrade
    if room.upgrade.is_some() {
        return Err("Room already has an upgrade! Remove it first.".into());
    }

    // Check costs
    if state.gold < template.gold_cost {
        return Err(format!("Not enough gold! Need {} gold.", template.gold_cost));
    }
    if state.souls < template.souls_cost {
        return Err(format!("Not enough souls! Need {} souls.", template.souls_cost));
    }

    // Deduct costs
    state.gold -= template.gold_cost;
    state.souls -= template.souls_cost;

    // Apply upgrade
    room.upgrade = Some(template.to_room_upgrade());

    state.add_log(LogEntry::building(format!(
        "Applied {} to floor {}, room {} for {} gold{}",
        upgrade_name,
        floor_num,
        room_pos,
        template.gold_cost,
        if template.souls_cost > 0 {
            format!(" and {} souls", template.souls_cost)
        } else {
            String::new()
        }
    )));

    Ok(())
}

/// Remove an upgrade from a room (no refund)
pub fn remove_upgrade(
    state: &mut GameState,
    floor_num: i32,
    room_pos: usize,
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

    // Check if room has an upgrade
    let upgrade_name = match &room.upgrade {
        Some(u) => u.name.clone(),
        None => return Err("Room has no upgrade to remove.".into()),
    };

    // Remove upgrade
    room.upgrade = None;

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

    // If room already has upgrade, return empty
    if room.upgrade.is_some() {
        return Ok(Vec::new());
    }

    // Return all upgrades - in future could filter by room type (boss gets better options)
    Ok(crate::data::get_all_upgrades())
}

/// Check if an upgrade can be afforded
pub fn can_afford_upgrade(state: &GameState, template: &UpgradeTemplate) -> bool {
    state.gold >= template.gold_cost && state.souls >= template.souls_cost
}

/// Get upgrade type icon/emoji
pub fn upgrade_type_icon(upgrade_type: &RoomUpgradeType) -> &'static str {
    match upgrade_type {
        RoomUpgradeType::Trap => "⚡",
        RoomUpgradeType::Treasure => "💰",
        RoomUpgradeType::Reinforcement => "🛡️",
        RoomUpgradeType::Evolution => "🧬",
    }
}
