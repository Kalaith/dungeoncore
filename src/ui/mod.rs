pub mod controls;
pub mod dungeon_view;
pub mod game_log;
pub mod monster_selector;
pub mod resource_panel;
pub mod species_selector;
pub mod upgrade_panel;

pub use controls::*;
pub use dungeon_view::*;
pub use game_log::*;
pub use monster_selector::*;
pub use resource_panel::*;
pub use species_selector::*;
pub use upgrade_panel::*;

// Layout constants
pub const PANEL_PADDING: f32 = 10.0;
pub const SIDEBAR_WIDTH: f32 = 250.0;
pub const TOP_BAR_HEIGHT: f32 = 60.0;
pub const LOG_HEIGHT: f32 = 150.0;
