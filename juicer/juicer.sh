#!/usr/bin/env bash

set -xe

# Sanity checks
if [[ ! -r "original.pdf" ]]; then
    echo "Missing original.pdf" >&2
    exit 1
fi

# Extract text from original PDF
pdftotext "original.pdf" "original.txt"

# Decide whether to enhance or not
if [[ "$(wc -c < "original.txt")" -lt 10 ]]; then
  echo "Document contains no text - enhancing" >&2
  "$(dirname "$0")/enhance.sh"
else
  # Just copy original PDF and already extracted text
  echo "Document already contains text" >&2
  cp 'original.pdf' 'document.pdf'
  cp 'original.txt' 'document.txt'
fi

# Extract preview
pdftoppm 'document.pdf' 'preview' -png -f 1 -singlefile

# Extract additional metadata
# This splits the pdfinfo output by line on first colon (':'), trims the values, filters for empty values and converts to JSON object
INFO="$(pdfinfo 'document.pdf' | jq --slurp --raw-input '
  split("\n") |
  map(
    split(":") |
    select(length >= 2) |
    {
      "key": .[0],
      "value": .[1:] | join(":") | sub("^ *"; "")
    } |
    select(.value != "")
  ) |
  from_entries
')"

META="$(cat "metadata.json")"

# Merge metadata
# The title will only be overridden if not set already, whereas the page count is always replaced
jq --slurp '
  . as [$info, $data] |
  $data * {
    "title": (($data | .["title"]) // ($info | .["Title"])),
    "pages": (($info | .["Pages"] | tonumber)),
  }
' <(echo "${INFO}") <(echo "${META}") >| "metadata.json"
