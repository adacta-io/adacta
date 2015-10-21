import uuid
from require import *

import bottle

from adacta.backend.web.api import resource
from adacta.backend.storage import Manifest



@resource('/bundles/<did>', ['GET'])
@require(storage='adacta.backend.storage:Storage')
def get_manifest(did,
                 storage):
    did = uuid.UUID(did)

    # Try to resolve the bundle
    try:
        bundle = storage.get(did)

    except FileNotFoundError:
        raise bottle.HTTPError(404, 'Bundle does not exist: %s' % did)

    # Load the manifest
    manifest = bundle.load_manifest()

    # Return the manifest as JSON
    return manifest.to_json()



@resource('/bundles/<did>', ['PUT'])
@require(storage='adacta.backend.storage:Storage',
         index='adacta.backend.index:Index',
         request='adacta.backend.web.api:Request')
def put_manifest(did,
                 storage,
                 index,
                 request):
    did = uuid.UUID(did)

    # Try to resolve the bundle
    try:
        bundle = storage.get(did)

    except FileNotFoundError:
        raise bottle.HTTPError(404, 'Bundle does not exist: %s' % did)

    # Ensure the document ID is not changed
    if 'did' in request.json:
        raise bottle.HTTPError(400, 'Document ID is altered')

    # Ensure the uploaded date is not changed
    if 'uploaded' in request.json:
        raise bottle.HTTPError(400, 'Uploaded date is altered')

    # Load the old manifest as JSON and update the manifest with the data from the request
    manifest = bundle.load_manifest().to_json()
    manifest.update(request.json)
    manifest = Manifest(manifest)

    # Save the manifest
    bundle.save_manifest(manifest=manifest)

    # Re-index the updated bundle
    index.index(bundle)

    # Return the manifest as JSON
    return manifest.to_json()



@resource('/bundles/<did>/fragments', ['GET'])
@require(storage='adacta.backend.storage:Storage')
def get_fragments(did,
                  storage):
    did = uuid.UUID(did)

    # Try to resolve the bundle
    try:
        bundle = storage.get(did)

    except FileNotFoundError:
        raise bottle.HTTPError(404, 'Bundle does not exist: %s' % did)

    # Return the list of fragments
    return {'did': str(did),
            'fragments': list(bundle)}



@resource('/bundles/<did>/fragments/<name>', ['GET'])
@require(storage='adacta.backend.storage:Storage')
def get_fragment(did,
                 name,
                 storage):
    did = uuid.UUID(did)

    # Try to resolve the bundle
    try:
        bundle = storage.get(did)

    except FileNotFoundError:
        raise bottle.HTTPError(404, 'Bundle does not exist: %s' % did)

    # Try to resolve the fragment
    if name not in bundle:
        raise bottle.HTTPError(404, 'Fragment does not exist: %s/%s' % (did, name))

    # Serve the fragment content
    # TODO: Don't allow access to whole system
    return bottle.static_file(filename=str(bundle[name]),
                              root='/')
