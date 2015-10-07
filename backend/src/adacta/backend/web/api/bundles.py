import uuid
from require import *

import bottle

from adacta.backend.web.api import resource



@resource('/bundles/<did>', ['GET'])
@require(storage='adacta.backend.storage:Storage')
def manifest(did,
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



@resource('/bundles/<did>/fragments', ['GET'])
@require(storage='adacta.backend.storage:Storage')
def fragments(did,
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
def fragment(did,
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
