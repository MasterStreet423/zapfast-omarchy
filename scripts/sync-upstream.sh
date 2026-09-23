#!/usr/bin/env bash
# Trae upstream/main a la rama actual y arregla solo los catálogos de gettext.
#
#   scripts/sync-upstream.sh           fetch + merge + catálogos
#   scripts/sync-upstream.sh --seguir  después de resolver a mano un conflicto de código
#
# zapfast.pot y es.po se generan desde el código, así que sus conflictos no se
# resuelven a mano: el .pot se regenera y en es.po gana nuestra traducción,
# sumando las que upstream haya agregado.
set -euo pipefail
cd "$(git rev-parse --show-toplevel)"

i18n=assets/i18n
otros=("$i18n"/de.po "$i18n"/fr.po "$i18n"/it.po "$i18n"/pt-BR.po "$i18n"/ru.po)

resolver_catalogos() {
    local conflictos
    conflictos=$(git diff --name-only --diff-filter=U)
    if grep -qx "$i18n/zapfast.pot" <<<"$conflictos"; then
        git checkout --theirs -- "$i18n/zapfast.pot"
    fi
    if grep -qx "$i18n/es.po" <<<"$conflictos"; then
        local tmp
        tmp=$(mktemp -d)
        git show ":2:$i18n/es.po" >"$tmp/nuestro.po"
        git show ":3:$i18n/es.po" >"$tmp/suyo.po"
        msgcat --use-first "$tmp/nuestro.po" "$tmp/suyo.po" -o "$i18n/es.po"
        rm -r "$tmp"
    fi
    for po in "${otros[@]}"; do
        if grep -qx "$po" <<<"$conflictos"; then
            git checkout --theirs -- "$po"
        fi
    done
    git add "$i18n" 2>/dev/null || true
}

regenerar() {
    .github/scripts/update-translations.sh >/dev/null
    # Los catálogos de otros idiomas quedan como los deja upstream.
    git checkout upstream/main -- "${otros[@]}"
    local faltan dudosas
    faltan=$(msgattrib --untranslated "$i18n/es.po" | grep -c '^msgid ' || true)
    dudosas=$(msgattrib --only-fuzzy "$i18n/es.po" | grep -c '^msgid ' || true)
    # La cabecera cuenta como un msgid vacío en la salida de msgattrib.
    if ((faltan > 0)); then faltan=$((faltan - 1)); fi
    if ((dudosas > 0)); then dudosas=$((dudosas - 1)); fi
    echo "es.po: $faltan sin traducir, $dudosas dudosas (fuzzy)"
}

if [[ "${1:-}" != --seguir ]]; then
    git fetch upstream
    if ! git merge --no-edit upstream/main; then
        resolver_catalogos
        pendientes=$(git diff --name-only --diff-filter=U)
        if [[ -n "$pendientes" ]]; then
            echo
            echo "Quedan conflictos de código; resuélvelos y corre: $0 --seguir"
            echo "$pendientes"
            exit 1
        fi
        git commit --no-edit
    fi
else
    if [[ -n "$(git diff --name-only --diff-filter=U)" ]]; then
        echo "Todavía hay conflictos sin resolver:" >&2
        git diff --name-only --diff-filter=U >&2
        exit 1
    fi
    git rev-parse -q --verify MERGE_HEAD >/dev/null && git commit --no-edit
fi

regenerar
# Si lo único que cambió es la fecha de extracción, no vale un commit.
if git diff -- "$i18n" | grep '^[-+][^-+]' | grep -qv 'POT-Creation-Date'; then
    git add "$i18n"
    git commit -q -m "Update the Spanish catalog after syncing upstream"
    echo "Catálogos actualizados en un commit aparte."
else
    git checkout -- "$i18n"
fi
