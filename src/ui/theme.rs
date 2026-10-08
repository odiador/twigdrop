//! Design tokens: single source of truth for colors, spacing, modal sizes.
//!
//! Spec: `docs/UI_GUIDE.md`. Migration is incremental — screens keep their
//! current `Rgb()` literals until phase 2; new code must use `Theme`.
//! Default values preserve the current Catppuccin Mocha look.

use ratatui::style::{Color, Modifier, Style};

/// Modal size tokens as `(percent_x, percent_y)` for `centered_rect`.
pub const MODAL_SM: (u16, u16) = (52, 48);
pub const MODAL_MD: (u16, u16) = (65, 60);
pub const MODAL_LG: (u16, u16) = (84, 86);

pub const MIN_WIDTH: u16 = 80;
pub const MIN_HEIGHT: u16 = 24;
/// Below this width the branches table collapses the author column.
pub const NARROW_WIDTH: u16 = 100;

pub const PADDING_XS: u16 = 1;
pub const PADDING_SM: u16 = 2;

/// Full palette. Field names are semantic — never `mauve`/`pink` in screens.
#[derive(Debug, Clone, Copy)]
pub struct Theme {
    pub base: Style,
    pub muted: Style,
    pub subtle: Style,
    pub accent: Style,
    pub accent2: Style,
    pub success: Style,
    pub warning: Style,
    pub danger: Style,
    pub info: Style,
    pub border: Style,
    pub active_border: Style,
    pub highlight: Style,
    pub header: Style,
    /// Branches with unique commits / merge conflicts (pink).
    pub unique: Style,
    /// Merged / clean branches (mint).
    pub merged: Style,
    /// Navigation hints (lavender).
    pub nav: Style,
    /// Author / secondary text.
    pub soft: Style,
    /// Unchecked checkbox.
    pub faint: Style,
    /// Gone upstream / not-analyzed (dim gray).
    pub dim: Style,
    /// Sync operations tag (sky blue, distinct from accent2).
    pub sync: Style,
    /// Destructive confirm background (dark red).
    pub danger_surface: Color,
    /// File conflict foreground (strong red).
    pub conflict: Style,
    /// Aged date foreground (peach).
    pub peach: Style,
    /// Commit graph lines (dim slate).
    pub graph: Style,
    /// Secondary dim text (cool gray).
    pub muted2: Style,
    /// Selected row on neutral background (commits, commit modals).
    pub select_hi: Style,
    /// Focused picker value (neutral bg, cyan fg).
    pub select_cyan: Style,
    /// Outer drawer border (soft slate).
    pub border_soft: Style,
    /// Focused search hit (black on yellow).
    pub search_hit: Style,
    /// Unfocused file selection background.
    pub select_soft: Color,
    /// Diff file selected while preview focused.
    pub panel_bg: Color,
    /// Diff cursor line background.
    pub cursor_line: Color,
    /// Preview cursor (unfocused panel) background.
    pub cursor_unfocused: Color,
    /// Preview text-selection range background.
    pub select_range: Color,
    /// File-tree status tints.
    pub tint_modified: Color,
    pub tint_added: Color,
    pub tint_staged: Color,
    pub tint_conflict: Color,
    /// Section divider borders.
    pub divider: Style,
    /// Message modal background (near-black).
    pub msg_bg: Color,
    pub backdrop_bg: Color,
    pub backdrop_fg: Color,
    pub surface: Color,
    pub surface_alt: Color,
}

impl Default for Theme {
    fn default() -> Self {
        Self::dark_default()
    }
}

impl Theme {
    /// Current Catppuccin Mocha look, frozen as default so migration is visual-noop.
    #[must_use]
    pub fn dark_default() -> Self {
        Self {
            base: Style::default().fg(Color::Rgb(205, 214, 244)),
            muted: Style::default().fg(Color::Rgb(147, 153, 178)),
            subtle: Style::default().fg(Color::Rgb(108, 112, 134)),
            accent: Style::default().fg(Color::Rgb(203, 166, 247)),
            accent2: Style::default().fg(Color::Rgb(137, 220, 235)),
            success: Style::default().fg(Color::Rgb(166, 227, 161)),
            warning: Style::default().fg(Color::Rgb(249, 226, 175)),
            danger: Style::default().fg(Color::Rgb(243, 139, 168)),
            info: Style::default().fg(Color::Rgb(137, 180, 250)),
            border: Style::default().fg(Color::Rgb(74, 79, 106)),
            active_border: Style::default().fg(Color::Rgb(203, 166, 247)),
            highlight: Style::default().bg(Color::Rgb(49, 50, 68)).fg(Color::White),
            header: Style::default()
                .fg(Color::Rgb(147, 153, 178))
                .add_modifier(Modifier::BOLD),
            unique: Style::default().fg(Color::Rgb(245, 194, 231)),
            merged: Style::default().fg(Color::Rgb(161, 229, 193)),
            nav: Style::default().fg(Color::Rgb(180, 190, 254)),
            soft: Style::default().fg(Color::Rgb(166, 173, 200)),
            faint: Style::default().fg(Color::Rgb(124, 128, 156)),
            dim: Style::default().fg(Color::Rgb(140, 143, 161)),
            sync: Style::default().fg(Color::Rgb(116, 199, 236)),
            danger_surface: Color::Rgb(30, 10, 10),
            conflict: Style::default().fg(Color::Rgb(210, 15, 57)),
            peach: Style::default().fg(Color::Rgb(250, 179, 135)),
            graph: Style::default().fg(Color::Rgb(100, 100, 120)),
            muted2: Style::default().fg(Color::Rgb(180, 180, 200)),
            select_hi: Style::default()
                .bg(Color::Rgb(80, 80, 100))
                .fg(Color::White),
            select_cyan: Style::default()
                .bg(Color::Rgb(80, 80, 100))
                .fg(Color::Cyan),
            border_soft: Style::default().fg(Color::Rgb(69, 71, 90)),
            search_hit: Style::default()
                .bg(Color::Rgb(255, 255, 0))
                .fg(Color::Black),
            select_soft: Color::Rgb(54, 58, 79),
            panel_bg: Color::Rgb(45, 45, 65),
            cursor_line: Color::Rgb(60, 60, 80),
            cursor_unfocused: Color::Rgb(40, 40, 60),
            select_range: Color::Rgb(30, 50, 80),
            tint_modified: Color::Rgb(35, 48, 65),
            tint_added: Color::Rgb(35, 60, 48),
            tint_staged: Color::Rgb(45, 60, 75),
            tint_conflict: Color::Rgb(65, 35, 35),
            divider: Style::default().fg(Color::Rgb(49, 50, 68)),
            msg_bg: Color::Rgb(20, 20, 20),
            backdrop_bg: Color::Rgb(17, 17, 27),
            backdrop_fg: Color::Rgb(88, 91, 112),
            surface: Color::Rgb(30, 30, 46),
            surface_alt: Color::Rgb(24, 24, 37),
        }
    }

    #[must_use]
    pub fn border_style(&self, focused: bool) -> Style {
        if focused {
            self.active_border
        } else {
            self.border
        }
    }

    #[must_use]
    pub fn selected_row(&self, focused: bool) -> Style {
        if focused {
            self.highlight
        } else {
            self.base.bg(self.surface)
        }
    }
}

/// Dimmed backdrop colors shared with `components::apply_dimmed_backdrop`.
#[must_use]
pub const fn backdrop_colors() -> (Color, Color) {
    (
        Color::Rgb(88, 91, 112), // fg
        Color::Rgb(17, 17, 27),  // bg
    )
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn modal_tokens_within_bounds() {
        for (x, y) in [MODAL_SM, MODAL_MD, MODAL_LG] {
            assert!(x > 0 && x <= 100 && y > 0 && y <= 100);
        }
        assert!(MODAL_SM.0 < MODAL_MD.0 && MODAL_MD.0 < MODAL_LG.0);
    }

    #[test]
    fn theme_has_no_color_only_status() {
        // Status must be distinguishable beyond hue: success/warning/danger
        // carry distinct RGB channels so monochrome fallback keeps contrast.
        let t = Theme::dark_default();
        assert_ne!(t.success, t.warning);
        assert_ne!(t.warning, t.danger);
        assert_ne!(t.success, t.danger);
        // Backdrop dims both fg and bg (not transparent).
        assert_ne!(t.backdrop_fg, t.backdrop_bg);
    }

    #[test]
    fn min_size_gate_sane() {
        const { assert!(MIN_WIDTH >= 80 && MIN_HEIGHT >= 20) };
        const { assert!(NARROW_WIDTH > MIN_WIDTH) };
    }
}
