#!/usr/bin/env bash

set -xe

if [[ ! -r "original.pdf" ]]; then
    echo "Missing original.pdf" >&2
    exit 1
fi

# Extract text from original PDF
pdftotext "original.pdf" "original.txt"

# Decide wether to enhance or not
if [[ "$(wc -c < "original.txt")" -lt 10 ]]; then
  "$(dirname "$0")/enhance.sh"
else
  # Just copy original PDF and already extracted text
  cp 'original.pdf' 'document.pdf'
  cp 'original.txt' 'document.pdf'
fi

# Extract thumbnails
convert -thumbnail x1024 'document.pdf' 'thumbnail-%03d.png'
