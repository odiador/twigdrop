# 🚀 Twigdrop Roadmap

## 🛡️ Seguridad y Prevención
- **Backup de SHAs:** Guardar SHAs en `~/.twigdrop/backups/<repo>/` antes de borrar para permitir restauraciones.
- **Ramas Protegidas:** Configuración en `~/.config/twigdrop/config.toml` (e.g., `protected = ["main", "master"]`).
- **Confirmación Crítica:** Modal de confirmación explícita para ramas con commits únicos (▲).

## 🔍 Funcionalidades de Navegación
- **Búsqueda Filtrada:** Modo `/` para filtrar la lista de ramas en tiempo real.
- **Ordenamiento Inteligente:** Tecla `s` para rotar entre nombre, recencia (comit) y status.
- **Columnas de Contexto:** Mostrar fecha del último commit y autor en la lista principal.

## 🗂️ Gestión de Ramas (Menú Manage)
- **Operaciones Expandidas:** Rename, Push, Set Upstream, Rebase/Merge.
- **Utilidades:** Copiar nombre al portapapeles y ver diff de stashes.

## 🎨 Experiencia de Usuario (UI/UX)
- **Diff Enriquecido:** Colores (Verde/Rojo) para sintaxis básica del diff.
- **Feedback Visual:** Spinner de carga y barra de estado dinámica.
- **Temas:** Soporte para modo claro/oscuro.

## 🔧 Excelencia Técnica
- **Robustez:** Reemplazar `.expect()` por gestión de errores `Result` en `git/commands.rs`.
- **Refactor:** Unificar constantes de layout entre `screens.rs` y `mouse.rs`.

---

### 📅 Plan de Acción Sugerido
1. **Seguridad:** Backups y protección de ramas.
2. **Core UX:** Búsqueda (`/`) y manejo de errores.
3. **Visual:** Columnas informativas y colores en Diff.

