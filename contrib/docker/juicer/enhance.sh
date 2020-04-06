#!/usr/bin/env bash

set -xe

# Get the scanned images out of the PDF
pdfimages 'original.pdf' 'raw'

# OCR each page
for ((I=0; 1; I++)); do
  SOURCE="$(printf 'raw-%03d.ppm' "${I}")"
  BETTER="$(printf 'better-%03d.ppm' "${I}")"
  TARGET="$(printf 'ocr-%03d' "${I}")"

  # Break on first non-existing file
  if [[ ! -r "${SOURCE}" ]]; then
    break
  fi

  # Enhance the page for OCR
  unpaper \
    --overwrite \
    "${SOURCE}" "${BETTER}"

  # OCR the page
  tesseract \
	  "${BETTER}" \
	  "${TARGET}" \
	  pdf
done

# Combine per page PDF files to a single one
pdfunite \
	ocr-???.pdf \
	'document.pdf'

# Extract the text of the final pdf file
pdftotext 'document.pdf' > 'document.txt'
