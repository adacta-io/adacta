import datetime

from require import *
from adacta.backend.web.api import resource



@resource('/upload', ['POST'])
@require(storage='adacta.backend.storage:Storage',
         index='adacta.backend.index:Index',
         request='adacta.backend.web.api:Request')
def upload(storage,
           index,
           request):

    # Get creation date from request
    uploaded = request.params.getone('uploaded',
                                     default=None,
                                     type=datetime.datetime.fromtimestamp)

    # Get tags from request
    tags = set(request.params.getall('tags'))

    # Create the bundle using all uploaded files
    bundle = storage.create(fragments={file.filename: file.file
                                       for file
                                       in request.files.values()},
                            uploaded=uploaded,
                            tags=tags)

    # Index the created bundle
    index.index(bundle)

    # return the bundles manifest as JSON
    return bundle.load_manifest().to_json()
