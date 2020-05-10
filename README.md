Adacta is a personal document archiving system.
It allows to categorize and organize PDF documents for long-term archiving as needed in personal document management.  

Its main features are a inbox concept allowing the user to review documents before archiving and a full-text search over all documents for easy retrial.
This is all archived while keeping a simple and documented on-disk format avoiding vendor-login and even encourages the usage of common filesystem utilities for mass-operations on documents.

Features
---
> **Note:** Not all features are implemented yet. 

Adacta concentrates on the following features:
* Documents, meta-data and all other permanent data is stored in a document repository as ordinary files and folders using common and widely adopted file formats.
  The filesystem structure and document-formats are well documented.
* State which is internal to Adacta can be rebuilt from the document repository.
  The only thing requiring backup is the repository.
* Uploaded PDF documents are OCRed if they do not contain extractable text.
  The original input document is stored beside the final document allowing to improve the OCR process after the documents have been archived.
  The whole OCR process is running in a docker container avoiding installing a complex and hard to maintain OCR software stack.
* Documents can be tagged.
  The tagging is aided by machine learning to suggest tags based on the documents text.

In contrast, Adacta declares the following non-features:
* No multi-user or mandate support and no ACL system.
  There may be multiple accounts to avoid sharing a password, but all users will always see the same data.
* No process management or document state / task tracking.
  This is pure archiving. While the inbox represents some kind of state, this state is only about the technical processing of the document.


Building
---
The [frontend](./frontend/README.md) must be build before the [backend](./backend/README.md).
See the `README.md` in the according folders for further instructions.


Running
---
Running Adacta requires a running [Docker](https://docker.com) daemon and an [Elasticsearch](https://elasti.co) cluster.

After building both, frontend and backend, the backend can be started by running
```
./backend/target/release/adacta --config path/to/your/adacta.yaml
```
