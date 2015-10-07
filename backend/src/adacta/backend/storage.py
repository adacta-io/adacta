from require import *

import uuid
import shutil
import pathlib
import datetime

from adacta.backend.manifest import Manifest



class Bundle(object):
    MANIFEST_FILENAME = 'manifest'


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
        return Manifest.load(path=self.path / Bundle.MANIFEST_FILENAME)


    def save_manifest(self, manifest):
        manifest.save(path=self.path / Bundle.MANIFEST_FILENAME)


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
               manifest,
               fragments):
        # Fill missing values in manifest
        manifest.setdefault('did', uuid.uuid4())
        manifest.setdefault('uploaded', datetime.datetime.now())

        # Build the manifest
        manifest = Manifest(manifest)

        # Build the path
        path = storage.path / str(manifest.did)

        # Create the bundle directory
        path.mkdir(parents=True)

        # Save the fragments
        for name, src in fragments.items():
            with (path / name).open(mode='wb') as dst:
                shutil.copyfileobj(src, dst)

        # Save the manifest
        manifest.save(path=path / Bundle.MANIFEST_FILENAME)

        # Create the bundle
        bundle = Bundle(storage=storage,
                        did=manifest.did)

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
               manifest,
               fragments):
        return Bundle.create(storage=self,
                             manifest=manifest,
                             fragments=fragments)


    def get(self, did):
        return Bundle(storage=self,
                      did=did)
