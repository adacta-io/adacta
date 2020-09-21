FROM jbarlow83/ocrmypdf:v11.1.0

RUN apt-get update && \
    apt-get install -y --no-install-recommends poppler-utils=0.86.1-0ubuntu1 tesseract-ocr-all=4.1.1-2build2 jq=1.6-1 && \
    rm -rf /var/lib/apt/lists/*

COPY juicer.sh enhance.sh /

RUN mkdir -p /juicer
WORKDIR /juicer

ENTRYPOINT ["/juicer.sh"]
