#!/usr/bin/env python3
"""Regenerates assets/i18n/emoji-<lang>.tsv from Unicode CLDR annotations.

Each line is `emoji<TAB>name|keyword|keyword...`, merging the base locale
with its regional override (es + es-419) and the derived annotations that
cover flags, keycaps and sequences. Skin-tone variants are dropped because
the picker searches base emoji only.

Usage: scripts/update-emoji-keywords.py [lang ...]   (default: es)
"""

import json
import sys
import urllib.request
from pathlib import Path

BASE = "https://raw.githubusercontent.com/unicode-org/cldr-json/main/cldr-json"
LOCALES = {"es": ["es", "es-419"]}
SKIN_TONES = {chr(c) for c in range(0x1F3FB, 0x1F400)}


def fetch(package: str, kind: str, locale: str) -> dict:
    url = f"{BASE}/{package}/{kind}/{locale}/annotations.json"
    with urllib.request.urlopen(url) as response:
        return json.load(response)[kind]["annotations"]


def merged(locales: list[str]) -> dict[str, list[str]]:
    # A regional locale overrides the name and the keywords separately: es-419
    # often redefines keywords only and inherits the name from es.
    fields: dict[str, dict[str, list[str]]] = {}
    for locale in locales:
        for package, kind in [
            ("cldr-annotations-full", "annotations"),
            ("cldr-annotations-derived-full", "annotationsDerived"),
        ]:
            for emoji, entry in fetch(package, kind, locale).items():
                if any(c in SKIN_TONES for c in emoji) or all(ord(c) < 0x2000 for c in emoji):
                    continue
                fields.setdefault(emoji, {}).update(
                    {key: entry[key] for key in ("tts", "default") if key in entry}
                )
    words: dict[str, list[str]] = {}
    for emoji, entry in fields.items():
        if "tts" not in entry:
            continue
        terms = entry["tts"] + entry.get("default", [])
        words[emoji] = list(dict.fromkeys(t.strip() for t in terms if t.strip()))
    return words


def main() -> None:
    root = Path(__file__).resolve().parent.parent
    for lang in sys.argv[1:] or ["es"]:
        words = merged(LOCALES.get(lang, [lang]))
        out = root / "assets" / "i18n" / f"emoji-{lang}.tsv"
        out.parent.mkdir(parents=True, exist_ok=True)
        lines = [f"{emoji}\t{'|'.join(terms)}" for emoji, terms in sorted(words.items())]
        out.write_text("\n".join(lines) + "\n", encoding="utf-8")
        print(f"{out.relative_to(root)}: {len(lines)} emoji")


if __name__ == "__main__":
    main()
