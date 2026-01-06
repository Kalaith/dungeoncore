pub mod resource_panel;
pub mod dungeon_view;
pub mod monster_selector;
pub mod game_log;
pub mod controls;
pub mod upgrade_panel;
pub mod species_selector;

pub use resource_panel::*;
pub use dungeon_view::*;
pub use monster_selector::*;
pub use game_log::*;
pub use controls::*;
pub use upgrade_panel::*;
pub use species_selector::*;

// Layout constants
pub const PANEL_PADDING: f32 = 10.0;
pub const SIDEBAR_WIDTH: f32 = 250.0;
pub const TOP_BAR_HEIGHT: f32 = 60.0;
pub const LOG_HEIGHT: f32 = 150.0;
