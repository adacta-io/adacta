from require import *



@export()
def Config():
    return {
        'storage': {
            'path': '/tmp/adacta/data'
        },
        'pipeline': {
            'path': '/tmp/adacta/pipeline'
        },
        'index': {
            'host': None
        }
    }
