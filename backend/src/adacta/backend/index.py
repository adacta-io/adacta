from require import *

import elasticsearch as es
import munch

from adacta.backend.storage import Bundle



@export()
class Index(object):
    log = require('adacta.backend.utils:Logger')

    config = require('adacta.backend.config:Config')

    ES_INDEX = 'documents'
    ES_TYPE = 'document'


    def __init__(self):
        # Open up elasticsearch connection
        # TODO: Support multiple hosts
        # TODO: Make sniffing configurable
        self.__es = es.Elasticsearch(hosts=self.config['index']['host'],
                                     sniff_on_start=True,
                                     sniff_on_connection_fail=True,
                                     sniffer_timeout=60)

        # Create the bundle index
        self.__es.indices.create(index=self.ES_INDEX,
                                 ignore=400)


    def index(self, bundle):
        # Load the manifest from bundle
        manifest = bundle.load_manifest()

        # Build the data to index by copying manifest
        data = munch.Munch(manifest.to_json())

        # Copy textual content if available
        if Bundle.FILENAME_DOCUMENT_TXT in bundle:
            with bundle.open(Bundle.FILENAME_DOCUMENT_TXT) as f:
                data.text = f.read()

        # Index the bundle
        self.log.debug('Indexing bundle %s: (%s)', bundle.did, data)
        self.__es.index(index=self.ES_INDEX,
                        doc_type=self.ES_TYPE,
                        body=data,
                        id=bundle.did)


    def search(self,
               body=Ellipsis,
               **kwargs):

        if body is Ellipsis:
            body = kwargs
        else:
            body.update(kwargs)

        return self.__es.search(index=self.ES_INDEX,
                                doc_type=self.ES_TYPE,
                                body=body)
