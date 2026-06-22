# basalto-tui

Interfaz de terminal (TUI) para el ecosistema Basalto. Navega, edita y usa tu biblioteca personal de archivos sin salir de la terminal.

## Características

- **Biblioteca** — navega carpetas y archivos con oil.nvim-style inline editing (`i`)
- **Usar archivo/plantilla** (`u`) — copia un archivo o carpeta completa a cualquier proyecto, con autocompletado de ruta con `Tab`
- **Metadata** — agrega descripción y tags a cada archivo (`:add`), consulta (`:show`), elimina del índice (`:remove`)
- **Editor integrado** — abre archivos directamente en tu editor de terminal (nvim, vim, nano), el TUI se suspende y se restaura al cerrar
- **Git** — vista de estado y log de la biblioteca con refresco en vivo
- **Plugins** — lista los plugins del ecosistema Basalto con su estado
- **Comandos** (`:`) — `add`, `show`, `remove`, `list`, `push`, `help`

## Instalación

```
cargo install --path .
```

Requiere que `basalto-core` esté instalado y configurado.

## Uso

```
basalto-tui
```

Abre el TUI en la biblioteca activa configurada en `~/.basalto/config.toml`.

## Atajos

| Tecla | Acción |
|---|---|
| `i` | Modo insertar — editar archivos inline |
| `u` | Copiar archivo o plantilla a un proyecto |
| `dd` | Marcar para eliminar |
| `Ctrl+S` | Confirmar cambios / eliminaciones |
| `j` / `k` | Navegar arriba y abajo |
| `h` / `l` | Cambiar tab |
| `:` | Abrir prompt de comandos |
| `?` | Ayuda |
| `q` | Salir |

### Copiar una plantilla

1. Navega a una carpeta guardada en la biblioteca (ej. `web/`)
2. Presiona `u`
3. Escribe la ruta destino o usa `Tab` para autocompletar
4. `Enter` para copiar

## Variables de entorno

| Variable | Descripción |
|---|---|
| `BASALTO_EDITOR` | Editor preferido (override explícito) |
| `EDITOR` / `VISUAL` | Editor del sistema (se ignoran editores GUI como `code`) |

Si ninguna variable está definida, basalto-tui detecta automáticamente `nvim`, `vim`, `nano` o `vi`.

## Licencia

MIT — ver [LICENSE](LICENSE).
