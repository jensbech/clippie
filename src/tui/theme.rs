use ratatui::style::Color;

// ── Palette — ALL explicit Rgb, zero ANSI named colors ─────
// Ported from ../scrn

pub const BASE_BG: Color = Color::Rgb(18, 18, 24);
pub const ZEBRA_BG: Color = Color::Rgb(30, 30, 40);
pub const HIGHLIGHT_BG: Color = Color::Rgb(55, 55, 80);

pub const FG: Color = Color::Rgb(220, 220, 230);
pub const DIM: Color = Color::Rgb(100, 100, 110);
pub const ACCENT: Color = Color::Rgb(180, 180, 255);
pub const HEADER_FG: Color = Color::Rgb(180, 180, 200);
pub const HELP_FG: Color = Color::Rgb(120, 120, 140);

pub const STATUS_OK: Color = Color::Rgb(140, 220, 140);
pub const STATUS_ERR: Color = Color::Rgb(220, 140, 140);
pub const MATCH_FG: Color = Color::Rgb(255, 200, 60);

pub const BORDER_FG: Color = Color::Rgb(60, 60, 80);
pub const MODAL_BG: Color = Color::Rgb(20, 20, 30);
pub const MODAL_BORDER: Color = Color::Rgb(80, 80, 110);
pub const MODAL_TITLE: Color = Color::Rgb(180, 180, 200);
pub const KILL_BG: Color = Color::Rgb(30, 15, 15);
pub const KILL_BORDER: Color = Color::Rgb(200, 80, 80);
pub const KILL_TITLE: Color = Color::Rgb(220, 140, 140);

pub const PATTERN_DATA: Color = Color::Rgb(140, 160, 220);
pub const PATTERN_EMAIL: Color = Color::Rgb(140, 220, 200);
pub const PATTERN_SECRET: Color = Color::Rgb(220, 140, 140);
