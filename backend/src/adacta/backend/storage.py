from require import *

import json
import uuid
import shutil
import pathlib
import datetime

from jsonobject import JsonObject
from jsonobject.properties import (DictProperty,
                                   JsonProperty,
                                   DateTimeProperty,
                                   SetProperty,
                                   StringProperty)



class UUIDProperty(JsonProperty):
    def wrap(self, obj):
        assert isinstance(obj, str)
        return uuid.UUID(obj)


    def unwrap(self, obj):
        assert isinstance(obj, uuid.UUID)
        return obj, str(obj)



class Manifest(JsonObject):
    uploaded = DateTimeProperty(default=datetime.datetime.now)
    """ The point in time, when the document was uploaded.
    """

    reviewed = DateTimeProperty(default=None)
    """ The point in time, when the document was last reviewed.
    """

    tags = SetProperty(StringProperty,
                       default=set)
    """ A set of tags assigned to the document
    """

    properties = DictProperty(StringProperty,
                              default=dict)
    """ Arbitrary properties assigned to the document
    """


    def save(self, path):
        """ Saves the manifest to the given path
            :param path: the path to save the manifest to
        """

        # Serialize the manifest directly to the file
        with path.open(mode='w') as f:
            json.dump(self.to_json(), f)


    @staticmethod
    def load(path):
        """ Loads a manifest from the given path
            :param path: the path to load the manifest from
            :return: the loaded manifest
        """

        # Deserialize the manifest from the file
        with path.open(mode='r') as f:
            return Manifest(json.load(f))



class Bundle(object):
    FILENAME_MANIFEST = 'manifest'
    FILENAME_DOCUMENT_PDF = 'document.pdf'
    FILENAME_DOCUMENT_TXT = 'document.txt'
    FILENAME_THUMBNAIL = 'thumbnail.png'
    FILENAME_LOG = 'log'


    log = require('adacta.backend.utils:Logger')


    def __init__(self,
                 storage,
                 did):
        self.__storage = storage
        self.__did = did

        if not self.path.exists:
            raise FileNotFoundError(self.path)


    @property
    def storage(self):
        return self.__storage


    @property
    def did(self):
        return self.__did


    @property
    def path(self):
        return self.storage.path / str(self.did)


    def load_manifest(self):
        return Manifest.load(path=self.path / Bundle.FILENAME_MANIFEST)


    def save_manifest(self, manifest):
        manifest.save(path=self.path / Bundle.FILENAME_MANIFEST)


    def delete(self):
        # Delete the bundle directory including content
        shutil.rmtree(self.path)


    def __getitem__(self, fragment):
        return self.path / fragment


    def __contains__(self, fragment):
        return self[fragment].exists()


    def __iter__(self):
        return (fragment.name
                for fragment
                in self.path.iterdir())


    def open(self, fragment):
        return self[fragment].open(mode='r')


    @classmethod
    def create(cls,
               storage,
               fragments,
               did=Ellipsis):

        if did is Ellipsis:
            did = uuid.uuid4()

        # Build the path
        path = storage.path / str(did)

        # Create the bundle directory
        path.mkdir(parents=True)

        # Save the fragments
        for name, src in fragments.items():
            with (path / name).open(mode='wb') as dst:
                shutil.copyfileobj(src, dst)

        # Ensure the bundle contains a correct manifest
        if Bundle.FILENAME_MANIFEST not in fragments:
            # Build a new manifest
            manifest = Manifest(uploaded=datetime.datetime.now())

            # Save the manifest
            manifest.save(path=path / Bundle.FILENAME_MANIFEST)

        # Create the bundle
        bundle = Bundle(storage=storage,
                        did=did)

        return bundle



@export()
class Storage(object):
    log = require('adacta.backend.utils:Logger')
    config = require('adacta.backend.config:Config')


    def __init__(self):
        if not self.path.exists():
            self.path.mkdir(parents=True)


    @property
    def path(self):
        return pathlib.Path(self.config['storage']['path'])


    def create(self,
               fragments,
               did=Ellipsis):
        return Bundle.create(storage=self,
                             fragments=fragments,
                             did=did)


    def get(self, did):
        return Bundle(storage=self,
                      did=did)
