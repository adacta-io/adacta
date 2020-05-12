#!/usr/bin/env bash

set -xe

# OCR the original pdf
ocrmypdf \
  -l eng+deu \
  --rotate-pages \
  --deskew \
  --remove-background \
  --clean \
  --output-type pdfa \
  --pdfa-image-compression jpeg \
  'original.pdf' \
  'document.pdf'

# Extract the text of the final pdf file
pdftotext 'document.pdf' > 'document.txt'
