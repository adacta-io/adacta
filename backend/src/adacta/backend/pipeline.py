from require import *
from abc import *

import uuid
import shlex
import pathlib
import threading
import subprocess
import collections

from adacta.backend.storage import Bundle


class Processor(object,
                metaclass=ABCMeta):
    log = require('adacta.backend.utils:Logger')
    storage = require('adacta.backend.storage:Storage')


    def __init__(self,
                 pipeline,
                 callback=None):
        self.__pipeline = pipeline
        self.__callback = callback

        if not self.path.exists():
            self.path.mkdir(parents=True)

        self.__signal = threading.Condition()

        self.__thread = threading.Thread(target=self.__run__,
                                         name='Processor: %s' % type(self))
        self.__thread.setDaemon(True)
        self.__thread.start()


    def __run__(self):
        self.log.debug('Processor %s started', self.name)

        while True:
            # Get all jobs waiting for the processor
            with self.__signal:
                jobs = list(self.path.iterdir())

            if jobs:
                self.log.debug('Processor %s got jobs: %s', self.name, jobs)

                # Process each job
                for job in jobs:
                    # Get the document ID from the job
                    did = uuid.UUID(job.name)

                    # Get the bundle instance for the document ID
                    bundle = self.storage.get(did)

                    # Process the bundle
                    try:
                        self._process(bundle)

                    except:
                        self.log.exception("Failed to process document %s", bundle.did)

                        continue

                    # Call the callback with the processed bundle
                    if self.__callback is not None:
                        self.__callback(bundle)

                    # Remove the symlink from the processors directory
                    with self.__signal:
                        job.unlink()

            # Wait for new jobs
            with self.__signal:
                self.__signal.wait(timeout=5.0)


    @abstractproperty
    def name(self):
        pass


    @abstractmethod
    def _process(self, bundle):
        pass


    def put(self, bundle):
        # Calculate the path in the processors directory
        path = self.path / str(bundle.did)

        with self.__signal:
            # Create a symlink to the bundles storage directory if it does not exist
            if not path.exists():
                path.symlink_to(bundle.path)

            # Notify the processor thread about the new data
            self.__signal.notify_all()


    @property
    def pipeline(self):
        return self.__pipeline


    @property
    def path(self):
        return self.pipeline.path / self.name


@export()
class Pipeline(object):
    log = require('adacta.backend.utils:Logger')
    config = require('adacta.backend.config:Config')


    def __init__(self):
        if not self.path.exists():
            self.path.mkdir(parents=True)

        self.__index_processors = IndexProcessor(self)
        self.__thumbnail_processors = ThumbnailProcessor(self, self.__index_processors.put)
        self.__text_processors = TextProcessor(self, self.__thumbnail_processors.put)


    @property
    def path(self):
        return pathlib.Path(self.config['pipeline']['path'])


    def put(self,
            bundle):
        self.__text_processors.put(bundle)


class AbstractCommandProcessor(Processor):
    @abstractproperty
    def command(self):
        pass


    def _process(self, bundle):
        command = self.command

        # Split the command if it's not a list of arguments
        if isinstance(command, str):
            command = shlex.split(self.command)

        # Open the log file
        with (bundle.path / Bundle.FILENAME_LOG).open(mode='at',
                                                      encoding='utf-8') as log:
            # Execute the command using the bundles directory as working directory and forward stdout and stderr to the
            # log file
            subprocess.run(command,
                           stdin=None,
                           stdout=log,
                           stderr=log,
                           cwd=str(bundle.path)).check_returncode()


class TextProcessor(AbstractCommandProcessor):
    @property
    def name(self):
        return 'text'


    @property
    def command(self):
        return 'pdftotext %s %s' % (Bundle.FILENAME_DOCUMENT_PDF,
                                    Bundle.FILENAME_DOCUMENT_TXT)


class ThumbnailProcessor(AbstractCommandProcessor):
    @property
    def name(self):
        return 'thumbnail'


    @property
    def command(self):
        return 'convert -thumbnail x1024 %s[0] %s' % (Bundle.FILENAME_DOCUMENT_PDF,
                                                      Bundle.FILENAME_THUMBNAIL)


class IndexProcessor(Processor):
    index = require('adacta.backend.index:Index')


    @property
    def name(self):
        return 'index'


    def _process(self, bundle):
        self.index.index(bundle)
