# zapfast-omarchy

Fork personal de [ZapFast](https://github.com/crmne/zapfast) que sigue al
oficial. `main` es un espejo puro de `crmne/zapfast`; `masterstreet` (la rama por
defecto) es la distribución: el oficial al día más estos cambios.

## Qué agrega

- **Interfaz entera en español.** Upstream traduce con gettext solo una parte
  de la interfaz; acá están marcados todos los textos y `assets/i18n/es.po`
  está completo. Lo que upstream guarda en constantes o en inglés en el
  archivo de mensajes (atajos, categorías de emoji, resúmenes como "Photo")
  se traduce al mostrarlo, en `src/i18n_extra.rs`. Los resúmenes ("Photo",
  "Voice message") los traduce upstream desde #527.
- **Emojis en español.** El selector y el autocompletado `:atajo:` buscan por
  los nombres y palabras clave de Unicode CLDR en español, sin importar las
  tildes (`corazon` encuentra ❤️). Tabla en `assets/i18n/emoji-es.tsv`,
  código en `src/emoji_words.rs`; se regenera con
  `scripts/update-emoji-keywords.py`.
- **Fondo de Omarchy en el chat.** Si el tema sigue al sistema en Omarchy, el
  fondo del escritorio se pinta detrás de la conversación, con un velo del
  color del chat para que se lea, y cambia en vivo con el fondo. Se apaga en
  Ajustes → Apariencia. Si se elige una imagen propia (lo que trae upstream
  desde 0.17), esa gana. Código en `src/desktop_background.rs`: el catálogo
  de temas vive en el crate `fastframe-theme` de upstream, así que el fondo
  no se mete en él; se recarga cada vez que el catálogo avisa un cambio.
- **Abrir chats desde el escritorio.** Mientras corre, ZapFast deja sus 60
  chats más recientes (sin bloqueados ni archivados) en
  `$XDG_RUNTIME_DIR/zapfast/chats.json`, legible solo por el usuario, y
  `zapfast open-chat <id>` abre uno en la instancia que está corriendo. Con
  eso un lanzador (el menú de Omarchy, por ejemplo) puede listar y abrir chats.
  Código en `src/chat_index.rs`. Convive con lo que upstream agregó en #426
  (`zapfast <número>`, `wa.me` y `whatsapp://`): eso abre por número; esto, por
  id, así que también abre grupos.

- **Responder desde la notificación.** El clic en la notificación de un mensaje
  abre una ventana chica con el mensaje, algunos de alrededor y un campo para
  contestar: Enter manda (citando el mensaje en grupos) y cierra, Esc cierra y
  deja el texto como borrador del chat, Ctrl+O abre el chat en la ventana
  completa. Con la app en la bandeja es la única ventana (app id
  `zapfast-reply`). El bloqueo de la app y los chats bloqueados mantienen el
  clic de siempre. Se apaga en Ajustes → Notificaciones. Código en
  `src/app/quick_reply.rs` y `src/ui/quick_reply.rs`.
- **Presencia manual.** La cuenta se muestra en línea según el foco de la
  ventana, no según lo que anuncia la librería por su cuenta; así, con
  ZapFast escondido en la bandeja, la cuenta no aparece en línea.
  `PresencePolicy::Manual` en `src/backend/worker.rs`.

- **Sin empaquetado de Windows ni winget.** El fork es para Omarchy; lo que
  upstream agregó en `packaging/windows/` y `packaging/winget/` no se trae.

## Traer lo nuevo del oficial

```bash
scripts/sync-upstream.sh           # fetch + merge + catálogos
scripts/sync-upstream.sh --seguir  # tras resolver a mano un conflicto de código
```

El README es el del fork: si choca, el script se queda con el nuestro.

`zapfast.pot` y `es.po` se generan desde el código, así que sus conflictos no
se resuelven a mano: el script regenera el `.pot` y, en `es.po`, deja nuestra
traducción y suma las nuevas de upstream. Al final dice cuántos textos
quedaron sin traducir o dudosos; esos se completan en `es.po`.

Si upstream agrega textos nuevos sin marcar, se marcan con
`crate::i18n::gettext(app.locale, "...")` como hace upstream y se traducen en
`es.po`.

## Compilar e instalar

```bash
cargo build --release
install -m755 target/release/zapfast ~/.local/bin/zapfast
```

Los tests corren siempre en inglés, sin importar el idioma del sistema.
