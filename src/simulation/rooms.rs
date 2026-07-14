use crate::data::constants::{get_room_cost, CORE_ROOM_MANA_BONUS, MAX_ROOMS_PER_FLOOR};
use crate::game_state::{Floor, GameState, LogEntry, Room, RoomType};

/// Add a room to the dungeon
pub fn add_room(state: &mut GameState, target_floor: Option<i32>) -> Result<(), String> {
    // Cannot add rooms while adventurers are in dungeon
    if !state.adventurer_parties.is_empty() {
        return Err("Cannot add rooms while adventurers are in the dungeon!".into());
    }

    // Find target floor
    let floor_num = target_floor.unwrap_or_else(|| {
        state
            .floors
            .iter()
            .find(|f| f.is_deepest)
            .map(|f| f.number)
            .unwrap_or(1)
    });

    let floor_idx = state
        .floors
        .iter()
        .position(|f| f.number == floor_num)
        .ok_or("Floor not found")?;

    // Count non-core rooms on this floor
    let non_core_count = state.floors[floor_idx]
        .rooms
        .iter()
        .filter(|r| r.room_type != RoomType::Core)
        .count();

    // Check if floor is full (entrance + 5 normal/boss rooms)
    if non_core_count > MAX_ROOMS_PER_FLOOR {
        // Create a new floor
        return create_new_floor(state);
    }

    // Calculate cost
    let total_rooms = state.total_room_count();
    let next_pos = non_core_count;
    let is_boss = next_pos == MAX_ROOMS_PER_FLOOR;
    let cost = get_room_cost(total_rooms, is_boss);

    if state.mana < cost {
        return Err(format!("Not enough mana! Need {} mana.", cost));
    }

    state.mana -= cost;

    // Create new room
    let room_type = if is_boss {
        RoomType::Boss
    } else {
        RoomType::Normal
    };
    let new_room = Room::new(
        macroquad_toolkit::rng::random_u64(),
        room_type.clone(),
        next_pos,
        floor_num,
    );

    // Insert before core room
    let floor = &mut state.floors[floor_idx];
    if let Some(core_idx) = floor
        .rooms
        .iter()
        .position(|r| r.room_type == RoomType::Core)
    {
        floor.rooms.insert(core_idx, new_room);
        // Update core room position
        floor.rooms[core_idx + 1].position = next_pos + 1;
    } else {
        floor.rooms.push(new_room);
    }
    // Extending the linear chain (Phase A): rewire exits in position order.
    // (Phase C's fork build op will wire branch edges explicitly instead.)
    floor.rebuild_linear_exits();

    let room_name = if is_boss { "Boss room" } else { "Normal room" };
    state.add_log(LogEntry::building(format!(
        "{} added to floor {} for {} mana.",
        room_name, floor_num, cost
    )));

    Ok(())
}

/// Create a new floor in the dungeon
fn create_new_floor(state: &mut GameState) -> Result<(), String> {
    let total_rooms = state.total_room_count();
    let cost = get_room_cost(total_rooms, false);

    if state.mana < cost {
        return Err(format!(
            "Not enough mana! Need {} mana to create new floor.",
            cost
        ));
    }

    state.mana -= cost;
    let new_floor_num = state.total_floors + 1;

    // Remove core from previous deepest floor
    for floor in &mut state.floors {
        if floor.is_deepest {
            floor.is_deepest = false;
            floor.rooms.retain(|r| r.room_type != RoomType::Core);
        }
    }

    // Create new floor
    let mut new_floor = Floor::new(macroquad_toolkit::rng::random_u64(), new_floor_num, true);
    new_floor.rooms.push(Room::new(
        macroquad_toolkit::rng::random_u64(),
        RoomType::Entrance,
        0,
        new_floor_num,
    ));
    new_floor.rooms.push(Room::new(
        macroquad_toolkit::rng::random_u64(),
        RoomType::Normal,
        1,
        new_floor_num,
    ));
    new_floor.rooms.push(Room::new(
        macroquad_toolkit::rng::random_u64(),
        RoomType::Core,
        2,
        new_floor_num,
    ));
    new_floor.rebuild_linear_exits();

    state.floors.push(new_floor);
    state.total_floors += 1;
    state.deep_core_bonus = state.total_floors as f32 * CORE_ROOM_MANA_BONUS;

    // A deeper core holds more mana — keeps late-tier summons affordable.
    state.max_mana += 50;

    state.add_log(LogEntry::building(format!(
        "New floor {} created for {} mana! Deep core bonus: +{}%, max mana {}",
        new_floor_num,
        cost,
        (state.deep_core_bonus * 100.0) as i32,
        state.max_mana
    )));

    Ok(())
}
