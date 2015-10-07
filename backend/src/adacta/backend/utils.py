from require import *

import logging


@export()
def Logger():
    logging.basicConfig(level=logging.DEBUG)

    logger = logging.getLogger('adacta')
    logger.setLevel(level=logging.DEBUG)

    return logger
