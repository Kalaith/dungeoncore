use macroquad::prelude::*;
use macroquad_toolkit::assets::AssetManager;
use macroquad_toolkit::input::{is_hovered_rect, was_clicked_rect};

use super::theme::*;

pub const TITLE_BACKGROUND_KEY: &str = "title_background";
pub const TITLE_BACKGROUND_PATH: &str = "assets/title_screen.png";

#[derive(Clone, Copy, Debug, PartialEq, Eq)]
pub enum TitleAction {
    None,
    NewGame,
    LoadGame,
    Settings,
    Exit,
}

#[derive(Clone, Copy, Debug, PartialEq, Eq)]
pub enum TitleSettingsAction {
    None,
    ToggleFullscreen,
    Back,
}

pub fn draw_title_screen(
    assets: &AssetManager,
    has_save: bool,
    notice: Option<&str>,
) -> TitleAction {
    let sw = screen_width();
    let sh = screen_height();
    draw_title_background(assets, sw, sh);

    let panel_w = sw.min(1280.0) * 0.29;
    let panel_w = panel_w.clamp(280.0, 380.0);
    let button_h = 48.0;
    let gap = 12.0;
    let panel_h = button_h * 4.0 + gap * 3.0 + 88.0;
    let x = (sw * 0.075).clamp(32.0, 118.0);
    let y = (sh - panel_h - 48.0).max(sh * 0.42);
    let panel = Rect::new(x, y, panel_w, panel_h.min(sh - y - 28.0));

    draw_title_panel(panel);
    draw_text_fit(
        "COMMAND THE CORE",
        panel.x + 26.0,
        panel.y + 31.0,
        panel.w - 52.0,
        15.0,
        Color::new(TEXT.r, TEXT.g, TEXT.b, 0.92),
    );
    let btn_x = panel.x + 22.0;
    let btn_w = panel.w - 44.0;
    let mut btn_y = panel.y + 82.0;

    if draw_title_button(
        Rect::new(btn_x, btn_y, btn_w, button_h),
        "New Game",
        true,
        ButtonTone::Arcane,
    ) {
        return TitleAction::NewGame;
    }

    btn_y += button_h + gap;
    if draw_title_button(
        Rect::new(btn_x, btn_y, btn_w, button_h),
        "Load Game",
        has_save,
        ButtonTone::Ghost,
    ) {
        return TitleAction::LoadGame;
    }

    btn_y += button_h + gap;
    if draw_title_button(
        Rect::new(btn_x, btn_y, btn_w, button_h),
        "Settings",
        true,
        ButtonTone::Ghost,
    ) {
        return TitleAction::Settings;
    }

    btn_y += button_h + gap;
    if draw_title_button(
        Rect::new(btn_x, btn_y, btn_w, button_h),
        "Exit",
        true,
        ButtonTone::Danger,
    ) {
        return TitleAction::Exit;
    }

    if !has_save {
        draw_text_fit(
            "No saved dungeon found.",
            btn_x,
            btn_y + button_h + 24.0,
            btn_w,
            12.0,
            TEXT_DIM,
        );
    }

    if let Some(message) = notice {
        draw_title_notice(message, sw, sh);
    }

    TitleAction::None
}

pub fn draw_title_settings_screen(
    assets: &AssetManager,
    fullscreen_enabled: bool,
    notice: Option<&str>,
) -> TitleSettingsAction {
    let sw = screen_width();
    let sh = screen_height();
    draw_title_background(assets, sw, sh);

    let panel_w = sw.min(1280.0) * 0.32;
    let panel = Rect::new(
        (sw - panel_w.clamp(320.0, 430.0)) * 0.5,
        (sh - 250.0) * 0.5,
        panel_w.clamp(320.0, 430.0),
        250.0,
    );
    draw_title_panel(panel);

    draw_text_fit(
        "SETTINGS",
        panel.x + 28.0,
        panel.y + 38.0,
        panel.w - 56.0,
        24.0,
        TEXT,
    );
    draw_text_fit(
        "Display preferences for this session.",
        panel.x + 28.0,
        panel.y + 64.0,
        panel.w - 56.0,
        13.0,
        TEXT_MUTED,
    );

    let button_w = panel.w - 56.0;
    let button_x = panel.x + 28.0;
    if draw_title_button(
        Rect::new(button_x, panel.y + 96.0, button_w, 48.0),
        if fullscreen_enabled {
            "Fullscreen: On"
        } else {
            "Fullscreen: Off"
        },
        true,
        ButtonTone::Arcane,
    ) {
        return TitleSettingsAction::ToggleFullscreen;
    }

    if draw_title_button(
        Rect::new(button_x, panel.y + 158.0, button_w, 48.0),
        "Back",
        true,
        ButtonTone::Ghost,
    ) || is_key_pressed(KeyCode::Escape)
    {
        return TitleSettingsAction::Back;
    }

    if let Some(message) = notice {
        draw_title_notice(message, sw, sh);
    }

    TitleSettingsAction::None
}

fn draw_title_background(assets: &AssetManager, sw: f32, sh: f32) {
    clear_background(BG_DEEP);

    if let Some(texture) = assets.get_texture(TITLE_BACKGROUND_KEY) {
        let scale = (sw / texture.width()).max(sh / texture.height());
        let draw_w = texture.width() * scale;
        let draw_h = texture.height() * scale;
        let x = (sw - draw_w) * 0.5;
        let y = (sh - draw_h) * 0.5;
        draw_texture_ex(
            texture,
            x,
            y,
            WHITE,
            DrawTextureParams {
                dest_size: Some(vec2(draw_w, draw_h)),
                ..Default::default()
            },
        );
    } else {
        draw_game_background(sw, sh);
    }

    draw_rectangle(0.0, 0.0, sw, sh, Color::new(0.0, 0.0, 0.0, 0.16));
    draw_rectangle(0.0, 0.0, sw * 0.42, sh, Color::new(0.0, 0.0, 0.0, 0.28));
    draw_rectangle(
        0.0,
        sh * 0.72,
        sw,
        sh * 0.28,
        Color::new(0.0, 0.0, 0.0, 0.34),
    );
}

fn draw_title_panel(rect: Rect) {
    draw_card(
        rect,
        Color::new(0.018, 0.020, 0.034, 0.72),
        Color::new(SOUL.r, SOUL.g, SOUL.b, 0.28),
    );
    draw_rectangle(
        rect.x,
        rect.y,
        3.0,
        rect.h,
        Color::new(SOUL.r, SOUL.g, SOUL.b, 0.68),
    );
}

fn draw_title_button(rect: Rect, text: &str, enabled: bool, tone: ButtonTone) -> bool {
    let hovered = enabled && is_hovered_rect(rect);
    let pressed = hovered && is_mouse_button_down(MouseButton::Left);
    let clicked = enabled && was_clicked_rect(rect);

    let (base, border, text_color) = match tone {
        ButtonTone::Primary => (EMERALD, EMERALD, TEXT),
        ButtonTone::Danger => (DANGER, DANGER, Color::new(1.0, 0.91, 0.91, 1.0)),
        ButtonTone::Ghost => (PANEL_ALT, BORDER, TEXT),
        ButtonTone::Arcane => (SOUL, SOUL, Color::new(0.96, 0.90, 1.0, 1.0)),
    };
    let alpha = if enabled {
        if pressed {
            0.34
        } else if hovered {
            0.28
        } else {
            0.18
        }
    } else {
        0.08
    };
    draw_card(
        rect,
        Color::new(base.r, base.g, base.b, alpha),
        Color::new(
            border.r,
            border.g,
            border.b,
            if hovered { 0.78 } else { 0.38 },
        ),
    );
    draw_centered_text(
        text,
        Rect::new(rect.x + 10.0, rect.y, rect.w - 20.0, rect.h),
        18.0,
        if enabled { text_color } else { TEXT_DIM },
    );

    clicked
}

fn draw_title_notice(message: &str, sw: f32, sh: f32) {
    let rect = Rect::new((sw - 430.0) * 0.5, sh - 76.0, 430.0, 42.0);
    draw_card(
        rect,
        Color::new(0.018, 0.020, 0.034, 0.82),
        Color::new(TREASURE.r, TREASURE.g, TREASURE.b, 0.32),
    );
    draw_centered_text(
        message,
        Rect::new(rect.x + 16.0, rect.y, rect.w - 32.0, rect.h),
        13.0,
        TEXT,
    );
}
