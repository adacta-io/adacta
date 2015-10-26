from require import *

import datetime
import uuid

from adacta.backend.web.api import resource



@resource('/upload', ['POST'])
@require(storage='adacta.backend.storage:Storage',
         pipeline='adacta.backend.pipeline:Pipeline',
         request='adacta.backend.web.api:Request')
def upload(storage,
           pipeline,
           request):

    # Get the predefined document ID to use (if any)
    did = request.params.getone('did',
                                default=Ellipsis,
                                type=uuid.UUID)

    # Create the bundle using all uploaded files
    bundle = storage.create(fragments={file.name: file.file
                                       for file
                                       in request.files.values()},
                            did=did)

    # Place the bundle in the pipeline
    pipeline.put(bundle)

    # return the bundles manifest as JSON
    return bundle.load_manifest().to_json()
