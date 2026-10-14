# 🚀 Twigdrop Roadmap

> Estado: actualizado tras las fases TUI/UX + seguridad. `[x]` = hecho y
> cubierto por tests; `[ ]` = pendiente.

## 🛡️ Seguridad y Prevención
- [x] **Backup de SHAs:** `actions::backup_branch_sha` guarda `<repo>/<fecha>.log` con `<hora> <rama> <sha>` en `~/.twigdrop/backups/<slug>/` antes de cada `branch -D` (best-effort, nunca bloquea el borrado).
- [x] **Ramas Protegidas:** `protected_branches = [...]` en `config.toml` se suma a los defaults (`Config::is_protected`); enforced en bulk-delete, prune, guards de UI y ordenación.
- [x] **Confirmación Crítica:** modal explícito para ramas con commits únicos (▲).

## 🔍 Funcionalidades de Navegación
- [x] **Búsqueda Filtrada:** type-to-filter en tiempo real + modal `/` legacy.
- [x] **Ordenamiento Inteligente:** `Ctrl+S` rota Recencia / Podables / A-Z.
- [x] **Columnas de Contexto:** fecha, autor, sync `↑↓` y salud de merge en la tabla (colapso responsive <100 cols).

## 🗂️ Gestión de Ramas (Menú Manage)
- [x] **Operaciones Expandidas:** Rename (modal dedicado `git branch -m`), Push, Set Upstream (`push -u`), Merge into current.
- [x] **Utilidades:** copiar nombre al portapapeles (arboard) y ver diff de stashes.
- [x] **Undo de borrado:** toast con `[u] undo` (8s) que recrea ramas desde los SHAs respaldados; cubierto por test E2E `delete_then_undo_restores_branch`.
- [ ] **Rebase/Merge desde Manage:** hoy vive en `Shift+R` (rebase interactivo); falta atajo directo merge-into-current.

## 🎨 Experiencia de Usuario (UI/UX)
- [x] **Diff Enriquecido:** syntax highlighting (syntect) + gutter `+/|/~`.
- [x] **Feedback Visual:** spinner braille en la barra de estado durante IA/animaciones (`App::is_busy`); empty-state diseñado; viewport `1-20/87`.
- [x] **Temas:** `dark` (Mocha) + `light` (Latte) vía `theme_mode` en config y selector en Settings.

## 🔧 Excelencia Técnica
- [x] **Robustez:** cero `.expect()`/`.unwrap()` en código de producción (`git/`, `actions/`); solo en tests.
- [x] **Refactor:** tokens únicos de layout (`ui::layout`) y color (`ui::theme::Theme`); `calculate_modal_rect` delega en `modal_size` (render y ratón coinciden).

---

### 📅 Plan de Acción Sugerido
1. **Seguridad:** Backups y protección de ramas. ✅
2. **Core UX:** Búsqueda y manejo de errores. ✅
3. **Visual:** Columnas informativas y colores en Diff. ✅
4. **Siguiente:** toast system con undo (`u`), tests de `draw` en modo claro.
