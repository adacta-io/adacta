
PAGES := $(shell pdfinfo | awk '/^Pages:/ { print $2; }')

.pages: original.pdf
	seq $$(pdfinfo $< | awk '/^Pages:/ { print $2; }' |) > $@

raw-%.ppm: original.pdf


enhanced-%.ppm: raw-%.ppm
	unpaper \
        --overwrite \
        "$<" "$@"

ocr-%.pdf: enhanced-%.ppm
	tesseract \
    	  "$<" "$@" \
    	  pdf

document.pdf: ocr
	pdfunite ocr-???.pdf 'document.pdf'

document.txt: document.pdf
	pdftotext 'document.pdf' 'document.txt'

thumbnails: document.pdf
	rm thumbnail-???.png
	convert -thumbnail x1024 'document.pdf' 'thumbnail-%03d.png'

all: document.pdf document.txt thumbnails
