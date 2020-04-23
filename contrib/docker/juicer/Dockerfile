FROM debian:buster

RUN apt-get update && apt-get install -y --no-install-recommends \
    make \
    gawk \
    unpaper \
    poppler-utils \
    imagemagick \
    ghostscript \
    tesseract-ocr-all

COPY juicer.sh enhance.sh /

RUN mkdir -p /juicer
#VOLUME /juicer
WORKDIR /juicer

ENTRYPOINT /juicer.sh
