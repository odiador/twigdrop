# Twigdrop — Canonical Architecture

> Standard: this doc is the single source of truth for module boundaries.
> Supersedes the legacy `ARCHITECTURE.md` at repo root (kept as stub).
> Aligned with `tui-design` skill pipeline: `Input → Event → State → Update → Layout → Render`.

## 1. Pipeline

```text
crossterm Event ──► src/events.rs (Event enum)
  ──► src/input/ (stateless KeyEvent → Action)
  ──► src/handlers/ (Action/View dispatch, calls update)
  ──► src/actions/* + src/state/* + src/git/* (update, I/O allowed)
  ──► src/ui/ (pure render: Fn(&App, Rect) — NO mutation)
  ──► ratatui Terminal (diff + present)
```

Async side-channel (never inside `draw`):
`src/tasks/* → tokio::mpsc → Event::Git | Event::Task → update`.

## 2. Module map

| Layer | Lives in | May do | Must NOT do |
|---|---|---|---|
| Events | `src/events.rs` | define `Event::{Key,Mouse,Resize,Git,Task}` | business logic |
| Input | `src/input/keybind.rs` | pure `KeyEvent → Option<Action>` via `(modifiers, code)` match | read `UiState`, store modifier bools |
| Handlers | `src/handlers/keyboard/*, mouse.rs` | route `Action` by current `View`/modal, call `App` update fns | hardcode `Rgb()` colors, duplicate shortcut tables |
| Update | `src/app.rs`, `src/actions/*`, `src/state/*`, `src/git/*` | mutate state, run git, push/pop modals | render widgets |
| Render | `src/ui/{mod,layout,theme,screens,components}.rs` | read `&App`, compute `Layout` from `area`, use `Theme` | mutate `App`, call git, spawn tasks |
| i18n | `src/i18n/*` | static `&'static str` catalogs | hold interaction state (e.g. shift/alt) |

## 3. State model (target — see debt note)

```text
View (exactly one): Branches | Files | Commits | Stashes
Overlay stack: Vec<Modal>  // Help, Manage, Filter, CommandPalette, ConfirmDelete, ...
Drawer flags: show_inspector_drawer, show_nav_sidebar (view-local, not modals)
```

* `View` ≠ `Modal`. Never push a view onto the modal stack.
* Exactly one focus at a time; modals trap focus until `Esc`/confirm.
* `Esc` is always `pop`, never implicit `push(MainMenu)`.
* Footer hints are static per `(View, Modal)` context — never reactive to a held modifier.

### Resolved (phases 1–4)

* ~~Triple source~~ → `View` + `AppMode::is_overlay` + `UiState::view()`; `alt/shift_pressed` deleted, modifiers matched per-event (`input::from_key`).
* ~~AppMode mezcla vistas~~ → variantes `*View` eliminadas; `mode_history: Vec<View>`, `SwitchTarget::ALL` único para switcher (render + teclado), `format_mode` muerto eliminado.
* ~~draw monolítico~~ → `draw` (update: caches + snap capture) + `render_views`/`render_chrome` (`&App`); `FrameCache` (RefCell) para scroll y hit-test de ratón.
* ~~Footer shift/alt~~ → hints estáticos por contexto en `FooterStrings` (`preview/diff/switcher/commit_files` + vistas).
* ~~Render mutation (scroll)~~ → `UiState::branch_viewport` (pura, testeada) + `FrameCache::sync_branches`.
* ~~Modal geometry drift~~ → `layout::modal_size` fuente única para render + ratón (`calculate_modal_rect` delega); tokens en `theme`/`layout`.
* ~~Colors/shortcuts scattered~~ → `ui::theme::Theme` (único sitio `Rgb()`, modos dark/light) + `input::BINDINGS` (help/footer consumen vía `keys()`).
* ~~git in render~~ → caches `inspector_*`/`worktrees` refrescados a la entrada de `draw` (inspector con gate por rama); corregidos los `"."` hardcodeados.

### Remaining debt (explicit, not hidden)

* Ninguna conocida en el pipeline de render. El capture del snap corre tras
  `render_views` por diseño (necesita el buffer pintado) pero el borrado
  completa siempre, con animaciones on/off (`snap_confirm_flow_deletes_without_animations`).

## 4. Input rules

* Match on `(key.modifiers, key.code)` per event — never on stored bools.
* `Shift+Tab` is `BackTab`, not `Tab+SHIFT`. `Shift+Letter` arrives as uppercase `Char` + `SHIFT`.
* Lone-`Esc` timeout respected; `Ctrl+C` reserved for quit (conflict: today `Ctrl+C` = create-branch — documented in `docs/KEYBINDINGS.md`, do not silently change).
* `?` toggles help; `Ctrl+K` palette; `/` filter/search (vim convention).

## 5. UI rules (summary — full in `docs/UI_GUIDE.md`)

* `Theme` only in `src/ui/theme.rs`. No `Rgb()` literals in screens.
* One `centered_rect` helper + `MODAL_SM/MD/LG` tokens. No ad-hoc centering.
* `Layout` derived from `f.area()` every frame; `MIN_WIDTH x MIN_HEIGHT` gate; truncate by display width with `…`.
* Never color-only status: icon + label (`✓ merged`, `⨯ gone`, `▲ unique`, `[P] protected`).

## 6. Testing contract

* `cargo fmt --check`, `cargo clippy --all-targets -- -D warnings`, `cargo test`.
* Unit: `input::keybind::from_key` mapping + `BINDINGS` coverage.
* Snapshot: `ratatui::backend::TestBackend` at fixed `120x30`, `draw → expected frame`.
* Interaction: scripted keys (`Down Down Enter`) → deterministic state. No TTY in CI.
