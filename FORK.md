# zapfast-omarchy

Fork personal de [ZapFast](https://github.com/crmne/zapfast) que sigue al
oficial. `main` es un espejo puro de `crmne/zapfast`; `masterstreet` (la rama por
defecto) es la distribución: el oficial al día más estos cambios.

## Qué agrega

- **Interfaz entera en español.** Upstream traduce con gettext solo una parte
  de la interfaz; acá están marcados todos los textos y `assets/i18n/es.po`
  está completo. Lo que upstream guarda en constantes o en inglés en el
  archivo de mensajes (atajos, categorías de emoji, resúmenes como "Photo")
  se traduce al mostrarlo, en `src/i18n_extra.rs`.
- **Emojis en español.** El selector y el autocompletado `:atajo:` buscan por
  los nombres y palabras clave de Unicode CLDR en español, sin importar las
  tildes (`corazon` encuentra ❤️). Tabla en `assets/i18n/emoji-es.tsv`,
  código en `src/emoji_words.rs`; se regenera con
  `scripts/update-emoji-keywords.py`.
- **Fondo de Omarchy en el chat.** Si el tema sigue al sistema en Omarchy, el
  fondo del escritorio se pinta detrás de la conversación, con un velo del
  color del chat para que se lea, y cambia en vivo con el fondo. Se apaga en
  Ajustes → Apariencia.
- **Abrir chats desde el escritorio.** Mientras corre, ZapFast deja sus 60
  chats más recientes (sin bloqueados ni archivados) en
  `$XDG_RUNTIME_DIR/zapfast/chats.json`, legible solo por el usuario, y
  `zapfast open-chat <id>` abre uno en la instancia que está corriendo. Con
  eso un lanzador (el menú de Omarchy, por ejemplo) puede listar y abrir chats.
  Código en `src/chat_index.rs`.

- **"En línea" solo cuando lo estás mirando.** whatsapp-rust anuncia presencia
  por su cuenta al conectar y cuando cambia el nombre, así que con la ventana
  escondida en la bandeja la cuenta podía quedar en línea. Acá la presencia es
  manual (`PresencePolicy::Manual` en `src/backend/worker.rs`) y la decide solo
  el foco de la ventana, como ya pretendía upstream.

## Traer lo nuevo del oficial

```bash
scripts/sync-upstream.sh           # fetch + merge + catálogos
scripts/sync-upstream.sh --seguir  # tras resolver a mano un conflicto de código
```

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

Se sacaron cinco tests de árabe y hebreo que fallan en Omarchy también sobre el
oficial limpio (dependen de las fuentes instaladas): cuatro en `src/bidi.rs`
(`arabic_lam_ligatures_…`, `ligatures_inside_right_to_left_…`,
`wrapped_right_to_left_…`, `message_bubbles_follow_the_bidi_…`) y
`rtl_self_chat_bubbles_render_like_whatsapp` en `src/demo.rs`. Si upstream los
toca, el merge choca ahí: se resuelve volviendo a sacarlos.
