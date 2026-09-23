# zapfast-omarchy

Fork personal de [ZapFast](https://github.com/crmne/zapfast) que sigue al
oficial. `main` es un espejo puro de `crmne/zapfast`; `erwin` (la rama por
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
