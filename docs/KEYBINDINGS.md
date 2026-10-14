# Twigdrop — Canonical Keybindings

> Source of truth: `src/input/keybind.rs` (`BINDINGS` + `from_key`).
> This doc mirrors that table. If they diverge, code wins and this doc must be regenerated.
> Conventions per `tui-design` skill `interaction-guide.md`.

## Rules

* Match on `(modifiers, code)` per `KeyEvent`. Never store `shift_pressed/alt_pressed` (`UiState` bools are deprecated, see `docs/ARCHITECTURE.md`).
* `Esc` is always pop/close — never implicit `push(MainMenu)`.
* `?` toggles help; `Ctrl+K` palette; `/`-style type-to-filter in Branches.
* Footer shows static per-context hints, not held-modifier state.

## Conflicts (explicit, do not silently change)

* `Ctrl+C` = create-branch today, but terminal convention reserves it for quit/interrupt. Candidate: move create to `Ctrl+N` or `n`. Kept as-is in skeleton phase.
* `Ctrl+S` / `Ctrl+Q` are flow-control keys when IXON is on; bound only because raw mode disables IXON. Document terminal fallback.
* `h` legacy help toggle collides with type-to-filter `h`; canonical is `?` / `Ctrl+H`.

## Global

| id | keys | i18n_key | description |
|---|---|---|---|
| global.quit | Ctrl+Q | common.quit | Quit safely |
| global.palette | Ctrl+K | modals.command_palette_title | Spotlight command palette |
| global.help_toggle_ctrl | Ctrl+H | modals.help_branch_ops_title | Toggle help |
| global.help_toggle_q | ? | modals.help_branch_ops_title | Toggle help (canonical) |
| global.close_back | Esc | common.close | Close modal / clear search / close drawer |
| global.toggle_view | Alt+D | modals.main_menu_app_switcher | Toggle Branches/Files view |
| global.switcher | Shift+Tab (BackTab key) | modals.main_menu_app_switcher | App view switcher |
| global.commits | Shift+C | modals.cmd_commit | Unpushed commits manager |
| global.stash_mgr | Shift+S | modals.cmd_stash | Stash manager |
| global.rebase | Shift+R | modals.cmd_rebase | Interactive rebase |
| global.quick_actions | : | modals.main_menu_quick_actions | Git quick actions palette |
| global.shell | ! | modals.cmd_commit | Shell command input |

## Branches

| id | keys | i18n_key | description |
|---|---|---|---|
| branches.navigate_up | Up / k | common.move_keys | Move selection up |
| branches.navigate_down | Down / j | common.move_keys | Move selection down |
| branches.toggle_select | Space | branches.header_sel | Toggle bulk selection |
| branches.manage | Enter | branches.manage_title | Manage branch (spotlight) |
| branches.type_filter | type [a-z0-9/_.-] | branches.type_to_filter | Instant filter |
| branches.undo | u (only while undo toast is live, else types `u`) | branches.toast_undo_hint | Restore last deleted branches |
| branches.clear_search | Esc (with query) | branches.esc_to_clear | Clear filter |
| branches.safe_delete | Ctrl+D | branches.act_delete_desc | Safe delete with confirm |
| branches.prune | Ctrl+P | branches.act_checkout_desc | Prune merged branches |
| branches.cycle_sort | Ctrl+S | branches.sort_label | Cycle Recent/Prunable/A-Z (⚠ flow-control when IXON on) |
| branches.create | Ctrl+C | modals.cmd_create_branch | Create branch (⚠ terminal convention reserves Ctrl+C for interrupt; candidate: Ctrl+N) |
| branches.filter | Ctrl+F | branches.header_sync | Filter-by-status modal |
| branches.inspector | Right | branches.inspector_hint | Inspector drawer |
| branches.sidebar | Left | branches.sidebar_hint | Nav sidebar |
| ai.analyze | Ctrl+I | branches.act_diff_ai_desc | AI analysis → Diff |

## Coverage check

`src/input/keybind.rs::bindings_table_covers_all_actions` asserts
`global.quit / global.palette / branches.safe_delete / branches.create / global.toggle_view`
exist and ids are unique. Extend that test when adding rows here.
