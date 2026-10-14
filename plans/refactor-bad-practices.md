# Plan: Malas prácticas y código muerto

Diagnóstico del 2026-10-13 (después de `54644d0`). Build, clippy (`-D warnings`) y los 69 tests están en verde.
Cada sección dice **qué hay**, **por qué es un problema** y **qué cambiar**. Hacerlas en orden; cada fase debe terminar con `cargo fmt && cargo clippy --all-targets -- -D warnings && cargo test` en verde y un commit propio.

---

## Fase 1 — Código muerto (`#[allow(dead_code)]`)

Regla general: **un `#[allow(dead_code)]` no debe existir en `main`**. O se conecta la funcionalidad o se borra (queda en el historial de git).

| Ubicación | Situación | Qué cambiar |
|---|---|---|
| [commands.rs](../src/actions/commands.rs) `abort_rebase`, `continue_rebase` | Existen pero ningún flujo los llama. Si un rebase interactivo falla, el usuario queda en estado `rebase-merge` sin salida desde la TUI. | **Conectar.** Cuando `is_rebasing()` sea `true`, mostrar un banner/toast con acciones "Continuar" / "Abortar" que llamen a estas funciones. Añadirlos como `CommandAction` en la paleta. |
| [status.rs](../src/git/status.rs) `is_merging`, `is_rebasing`, `get_conflicting_files` | Ya tienen tests, pero nadie los usa en runtime. | **Conectar** junto con lo anterior: llamarlos en `refresh_branches` y guardar un `RepoState { Clean, Merging, Rebasing { conflicts } }` en `App`. Mostrarlo en el sidebar/statusbar. |
| [models.rs](../src/models.rs) `MergeStatus::Checking` | Variante nunca construida: el análisis de merge corre async pero la UI no muestra estado "analizando". | **Conectar:** asignar `Checking` al despachar el análisis en `runtime.rs` y renderizar un spinner en branches. Si no se quiere, **borrar** la variante. |
| [git/mod.rs](../src/git/mod.rs) `get_branch_commits` | Duplica la lógica de `refresh_inspector_commits` en `app.rs` (que hace su propio `git log -n 4`). | **Unificar:** que el inspector use `get_branch_commits(path, branch, limit)` (agregar parámetro `limit`) y borrar el `run_git` inline de `app.rs:373`. |
| [actions/mod.rs](../src/actions/mod.rs) `ActionId` + `#[allow(unused_imports)] apply_resolution_to_file` | `ActionId` es un enum sin uso (había sido pensado para keybinds); el re-export no se usa. | **Borrar** `ActionId` (el sistema real está en `input/keybind.rs`). Borrar el re-export; quien lo necesite usa `actions::commands::apply_resolution_to_file`. |

---

## Fase 2 — God files

Umbral orientativo: **>400 líneas o más de una responsabilidad** → dividir.

### 2.1 `src/handlers/keyboard/modals.rs` (933 líneas) — prioridad alta
**Problema:** un único `match` gigante maneja el input de *todos* los modales (paleta, quick actions, confirmaciones, inputs de crear/renombrar branch, rebase, settings...) y además ejecuta git directamente.
**Qué cambiar:**
- Crear `src/handlers/keyboard/modals/` con un archivo por modal, espejando `src/ui/screens/modals/`:
  `command_palette.rs`, `quick_actions.rs`, `confirm.rs`, `input.rs` (create/rename), `rebase.rs`, `settings.rs`, `date_picker.rs`, `main_menu.rs`.
- `modals/mod.rs` queda solo como dispatcher: `match app.ui.current_mode() { AppMode::X => x::handle(app, key, path), ... }`.
- Cada handler debe devolver una intención (`Option<CommandAction>`) en vez de ejecutar git (ver Fase 3).

### 2.2 `src/app.rs` (785 líneas) — prioridad alta
**Problema:** `impl App` mezcla navegación, carga de datos git, árbol de archivos, stashes, rebase, AI, syntax highlighting de previews y lógica de borrado/undo.
**Qué cambiar** (mover métodos a módulos `impl App` separados, sin cambiar la API pública al principio):
- `src/app/mod.rs` → struct `App`, `new`, `update`, `locale`, `theme`, `is_busy`.
- `src/app/branches.rs` → `refresh_branches`, `cycle_branch_sort`, `refresh_filtered_branches`, `get_filtered_branches`, `next/previous`, `toggle_selection`.
- `src/app/deletion.rs` → `snapshot_shas`, `delete_branches_by_name`, `complete_snap_deletion`, `undo_delete`.
- `src/app/files.rs` → `load_file_tree`, `build_tree_recursive`, `toggle_file_dir` (+ quitar el parámetro `_path_str` sin uso).
- `src/app/preview.rs` → `create_preview_state`, `update_preview_highlighting`, `update_diff_highlighting`.
- `src/app/stash_rebase.rs` → `load_stashes`, `load_stash_detail`, `load_rebase_commits*`.
- `src/app/ai.rs` → `AIState`, `setup_ai` (quitar `_path` sin uso), `mark_ai_dispatched`.

### 2.3 `src/state/ui.rs` (578 líneas)
**Problema:** contiene ~20 tipos de estado de todos los modales más `UiState` y `FrameCache`.
**Qué cambiar:**
- `src/state/modals.rs` → `DatePickerState`, `CommandPaletteState`, `SettingsState`, `MainMenuState`, `QuickActionsState`, `ModalState`.
- `src/state/rebase.rs` → `RebaseCommit`, `RebaseAction`, `RebaseState`.
- `src/state/frame_cache.rs` → `FrameCache`.
- `ui.rs` queda con `UiState`, `AppMode`, `View`, `PrimaryMode`, `SwitchTarget`.

### 2.4 `src/ui/screens/branches.rs` (619) y `src/ui/mod.rs` (571)
- `branches.rs`: separar la tabla, el empty state y el panel de merge status en `branches/{table,empty,merge_status}.rs`.
- `ui/mod.rs`: los ~250 líneas de tests de render (con `Terminal::new(...).unwrap()` repetido 9 veces) → mover a `src/ui/tests.rs` con un helper `fn render(app: &mut App) -> String`.

---

## Fase 3 — Git fuera de la capa de handlers/app

**Problema:** hay `run_git(...)` crudo fuera de `src/git` y `src/actions`:
`handlers/keyboard/modals.rs` (11), `handlers/keyboard/commits.rs` (5), `app.rs` (4). Ejemplos: `checkout`, `push origin`, `push -u origin`, `merge`, `checkout -b`, `branch -m`, `commit --fixup`, `rev-parse`. Esto duplica lógica, hace imposible testearla y mezcla los mensajes de error con la UI. Además `commits.rs:66` descarta el resultado con `let _ =` (fallo silencioso).
**Qué cambiar:**
- Crear funciones tipadas en `src/actions/commands.rs` (o `src/git/ops.rs`): `checkout(path, branch)`, `create_branch`, `rename_branch`, `push_branch(path, branch, set_upstream)`, `merge_branch`, `fixup_commit`, `resolve_full_hash`, `commit_date`, `list_worktrees`.
- Todas devuelven `Result<String, GitError>`; los handlers solo llaman a la función y muestran un toast.
- Prohibir `run_git` fuera de `src/git`/`src/actions`: hacerlo `pub(crate)` dentro de `git` y re-exportar solo las funciones de alto nivel.
- Reemplazar `let _ = run_git(...)` por manejo explícito + toast de error.

---

## Fase 4 — Magic strings y números

### 4.1 Branches sintéticas `"*Local Changes*"` / `"*Staged Changes*"`
**Problema:** 10 apariciones en `src/git/mod.rs` comparando strings; un typo rompe la lógica sin error de compilación, y una branch real podría llamarse así.
**Qué cambiar:** introducir en `models.rs`
```rust
pub enum BranchKind { Real, LocalChanges, StagedChanges }
```
como campo de `Branch` (o constantes `pub const LOCAL_CHANGES: &str` como paso intermedio). Las comparaciones pasan a `matches!(b.kind, BranchKind::LocalChanges)`. El texto visible sale de `i18n`.

### 4.2 Remote `"origin"` hardcodeado
**Problema:** `push origin` / `push -u origin` en `modals.rs:420-428` fallan en repos cuyo remoto no se llama `origin`.
**Qué cambiar:** resolver el remoto con `git config branch.<b>.remote`, y si no hay, el primero de `git remote`; opcionalmente `default_remote` en `utils/config.rs`.

### 4.3 Lista de branches protegidas
`actions/commands.rs:6` tiene `["main","master","develop","dev","staging","production"]` y `utils/config.rs` también tiene `is_protected`. **Unificar:** que la constante se llame `DEFAULT_PROTECTED_BRANCHES`, viva en `utils/config.rs` y sea el default del config; `commands.rs` consulta solo `config.is_protected()`.

### 4.4 Formatos de `git log`
`"--format=%h|%s|%cr|%an"`, `"--format=%h %s"`, `"--format=%cI"` repetidos inline. **Mover** a constantes en `src/git/formats.rs` junto al parser que los consume (formato y parser deben cambiar juntos).

### 4.5 Tiempos y tamaños
`main.rs`: `from_millis(10)` (poll), `16` (frame ~60fps), `250` (tick); `runtime.rs:69`: `from_secs(2)`; `app.rs:373`: `-n 4`. **Pasar a constantes con nombre** (`INPUT_POLL`, `FRAME_INTERVAL`, `TICK_INTERVAL`, `REFRESH_DEBOUNCE`, `INSPECTOR_COMMIT_LIMIT`) en un `src/constants.rs` o al tope de cada módulo, como ya se hace en `snap.rs` y `toast.rs`.

### 4.6 Mensajes de error/usuario en inglés hardcodeados
`commands.rs` tiene `format!("Rebase aborted successfully...")`, `"Failed to ..."`, etc., ignorando `i18n`. **Qué cambiar:** las acciones devuelven un `GitError`/`ActionOutcome` enum; el texto se genera en la UI vía `app.locale()`. Agregar las claves en `i18n/en.rs` y `i18n/es.rs`.

### 4.7 Colores
Verificar que no queden `Color::Rgb(..)` / `Color::X` fuera de `ui/theme.rs` (`grep -rn "Color::" src/ui | grep -v theme.rs`). Todo color debe venir de `app.theme()`.

---

## Fase 5 — Errores tipados

**Problema:** las funciones devuelven `Result<String, String>`; no se puede distinguir "no hay remoto" de "conflicto" de "git no instalado".
**Qué cambiar:** crear `src/git/error.rs`:
```rust
#[derive(Debug, thiserror::Error)]
pub enum GitError {
    #[error("git command failed: {0}")] Command(String),
    #[error("no remote configured")] NoRemote,
    #[error("merge conflict")] Conflict(Vec<String>),
    #[error(transparent)] Io(#[from] std::io::Error),
}
```
Migrar `run_git` y las funciones de Fase 3 a `Result<_, GitError>`. Hacerlo después de Fase 3 para no tocar dos veces los mismos call-sites.

---

## Fase 6 — Lints más estrictos (opcional, al final)

`clippy::pedantic` reporta ~260 warnings (`#[must_use]`, or-patterns anidados, casts `usize→u16` que pueden truncar, `let...else`).
**Qué cambiar:**
- Arreglar primero los que son bugs potenciales: `cast_possible_truncation` (6) — usar `u16::try_from(x).unwrap_or(u16::MAX)`.
- Luego activar en `Cargo.toml`:
```toml
[lints.clippy]
pedantic = { level = "warn", priority = -1 }
module_name_repetitions = "allow"
missing_errors_doc = "allow"
```
- Añadir `cargo fmt --check` y `cargo clippy -- -D warnings` al `Makefile` (target `check`) y a CI.

---

## Checklist

- [ ] Fase 1 — código muerto conectado o eliminado (0 `#[allow(dead_code)]`)
- [ ] Fase 2 — ningún archivo >400 líneas con responsabilidades mezcladas
- [ ] Fase 3 — 0 `run_git` fuera de `src/git` / `src/actions`
- [ ] Fase 4 — 0 magic strings de branches sintéticas, remote, formatos y tiempos
- [ ] Fase 5 — `GitError` tipado
- [ ] Fase 6 — pedantic + CI
