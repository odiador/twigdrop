# Twigdrop — UI Guide

> Tokens: `src/ui/theme.rs`. Bindings: `src/input/keybind.rs` + `docs/KEYBINDINGS.md`.
> Reference: `tui-design` skill (`design-principles`, `component-catalog`, `states-and-feedback`).

## 1. Theme — always use tokens

```rust
use crate::ui::theme::Theme;
let t = Theme::dark_default();
Block::default().border_style(t.border_style(focused));
```

* No `Color::Rgb(..)` literals in `screens/` or `components.rs` for new code.
* Semantic names only: `success/warning/danger/info/accent/muted/subtle`.
* Status is never color-only: `✓ merged · ⨯ gone · ▲ unique · [P] protected` + text label.

## 2. Layout

* Every frame derives from `f.area()`. Never hardcode terminal size.
* Modals via `components::centered_rect` with tokens:
  `MODAL_SM (52,48)` Manage/palette · `MODAL_MD (65,60)` confirm/settings · `MODAL_LG (84,86)` help.
* Minimum gate: `MIN_WIDTH x MIN_HEIGHT = 80x24` → polite message, re-check on `Resize`.
* Narrow mode: below `NARROW_WIDTH (100)` the branches table collapses the author column and abbreviates sync text. Truncate by display width with `…`, pad width-aware.

## 3. Branches table (scope of this phase)

* Columns desktop: `sel | branch+icons | sync | merge-health | age | last-commit`.
* Narrow: hide `last-commit`, keep `branch + sync + merge`.
* Viewport indicator top-right (`1-20 of 87`) stays; add scrollbar on phase 2.
* Empty state (required phase 2): `No branches match → [Ctrl+C] create · [Esc] clear filter`.
* Scroll math moves to update (`next/previous/refresh_filtered_branches`); render takes `&App` and never mutates `list_start_index`.

## 4. Modals

* All modals dim with `backdrop_colors()` (`fg 88,91,112 / bg 17,17,27`) then `Clear` the centered rect.
* Spotlight pattern (palette + manage): header search bar `Length(3)`, list `Min(6)`, footer hints `Length(1)`.
* Footer hints static per context: `↑↓ select · ⏎ execute · Esc cancel`. Never depend on held Shift/Alt.
* Destructive default is Cancel; destructive option visually distinct (`danger`).

## 5. States & feedback

* Loading: spinner in status bar (`⠋`) for fetch/pull/AI; never freeze without indicator.
* Toast (replaces `Message(String)` in phase 2): `{kind: info|success|error, text, ttl: 2-3s}` + `[u] undo` for deletes.
* Error views show next action (`press r to retry`), never bare panic text.
* Respect `NO_COLOR` / `TERM=dumb`: fall back to bold/reverse, no emoji-dependent meaning.

## 6. Implementation checklist (Branches + global modals)

* [ ] New render code uses `Theme` + `MODAL_*`.
* [ ] New bindings added to `BINDINGS` + `docs/KEYBINDINGS.md` + footer/help.
* [ ] `cargo fmt --check`, `clippy --all-targets`, `cargo test` green.
* [ ] `TestBackend 120x30` snapshot for Branches normal + palette overlay + narrow `78x24` gate.
