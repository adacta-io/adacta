FROM jbarlow83/ocrmypdf

RUN apt-get update && apt-get install -y --no-install-recommends \
    poppler-utils \
    tesseract-ocr-all

COPY juicer.sh enhance.sh /

RUN mkdir -p /juicer
WORKDIR /juicer

ENTRYPOINT /juicer.sh
