<p align="center">
  <img src="docs/assets/images/logo.svg" alt="Logo de ZapFast" width="88" height="88">
</p>

<h1 align="center">zapfast-omarchy</h1>

<p align="center"><strong>ZapFast en español y hecho para Omarchy.</strong><br>
El cliente de WhatsApp nativo de <a href="https://github.com/crmne/zapfast">crmne/zapfast</a>, al día con el oficial, más lo que le falta en un escritorio Omarchy.</p>

<p align="center">
  <a href="https://github.com/MasterStreet423/zapfast-omarchy/releases/latest"><strong>Descargar</strong></a> ·
  <a href="FORK.md">Cómo está hecho</a> ·
  <a href="https://github.com/crmne/zapfast">ZapFast oficial</a>
</p>

## Qué agrega

| | |
|---|---|
| **Interfaz entera en español** | Todos los textos marcados con gettext y `es.po` completo, incluidos los que el oficial deja fijos en el código (atajos, categorías de emoji). |
| **Emojis en español** | El selector y el autocompletado `:atajo:` buscan por los nombres CLDR en español, sin importar las tildes: `corazon` encuentra ❤️. |
| **Responder desde la notificación** | Un clic en la notificación abre una ventana chica para contestar. Enter manda, Esc deja el texto como borrador, Ctrl+O abre el chat completo. |
| **Fondo de Omarchy en el chat** | Si el tema sigue al sistema, el fondo del escritorio se ve detrás de la conversación y cambia en vivo con él. |
| **Abrir chats desde el escritorio** | `zapfast open-chat <id>` y la lista de chats recientes en `$XDG_RUNTIME_DIR/zapfast/chats.json`, para que un lanzador como el menú de Omarchy los liste y los abra, grupos incluidos. |

Todo lo demás —mensajes, llamadas, varias cuentas, archivo cifrado, temas— es
el ZapFast oficial tal cual. El detalle de cada cambio, dónde vive en el código
y cómo se trae lo nuevo del oficial está en [FORK.md](FORK.md).

## Instalar

Bajar el último build de [Releases](https://github.com/MasterStreet423/zapfast-omarchy/releases/latest)
e instalarlo en `~/.local/bin`:

```sh
curl -L https://github.com/MasterStreet423/zapfast-omarchy/releases/latest/download/zapfast-omarchy-x86_64-linux.tar.gz | tar xz
install -Dm755 zapfast-omarchy-*/zapfast ~/.local/bin/zapfast
```

`~/.local/bin` va antes que `/usr/bin` en el PATH de Omarchy, así que tapa a un
`zapfast-bin` instalado con pacman sin desinstalarlo. Este build no se
actualiza solo: el autoactualizador de ZapFast bajaría el oficial, en inglés y
sin estos cambios.

### Compilar

```sh
cargo build --release
install -Dm755 target/release/zapfast ~/.local/bin/zapfast
```

Las dependencias del sistema son las del oficial: ver
[Getting started](https://zapfast.rocks/getting-started/).

## Vincular el teléfono

Al abrirlo muestra un código QR. En el teléfono: WhatsApp → **Ajustes** →
**Dispositivos vinculados** → **Vincular un dispositivo**, y escanear. También
se puede vincular con un código de ocho letras desde **Vincular con número de
teléfono**.

## Créditos y aviso

Todo el trabajo de fondo es de [ZapFast](https://github.com/crmne/zapfast), de
Carmine Paolino y sus colaboradores, sobre
[whatsapp-rust](https://github.com/oxidezap/whatsapp-rust),
[egui](https://www.egui.rs) y [fastframe](https://fastframe.dev). Las fuentes,
íconos y sonidos que trae tienen sus propias licencias, listadas en el
[README del oficial](https://github.com/crmne/zapfast#acknowledgements).

ZapFast es un cliente no oficial y no tiene relación con WhatsApp ni con Meta.
Usar un cliente no oficial puede ir contra los términos de servicio de
WhatsApp y podría hacer que suspendan la cuenta: úsalo bajo tu propio riesgo.
WhatsApp es una marca registrada de WhatsApp LLC.

Licencia [MIT](LICENSE), heredada del oficial.
