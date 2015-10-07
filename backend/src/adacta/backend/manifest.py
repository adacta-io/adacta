import json
import uuid
import datetime

from jsonobject import JsonObject
from jsonobject.properties import (
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

    did = UUIDProperty(required=True)
    """ The document ID.
    """

    uploaded = DateTimeProperty(required=True)
    """ The point in time, when the document was uploaded.
    """

    reviewed = DateTimeProperty()
    """ The point in time, when the document was last reviewed.
    """

    tags = SetProperty(StringProperty)
    """ A set of tags assigned to the document
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
