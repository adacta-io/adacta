from require import *



@export()
def Config():
    return {
        'storage': {
            'path': '/tmp/adacta/data'
        },
        'index': {
            'host': None
        }
    }
