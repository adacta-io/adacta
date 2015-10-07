from require import *

import bottle



@export()
def App():
    # Create a new bottle application
    return bottle.Bottle()



def route(path,
          methods='GET'):
    @require(app='adacta.backend.web:App',
             logger='adacta.backend.utils:Logger')
    def extender(func,
                 app, logger):
        logger.debug('Register route: %s %s -> %s',
                     path, methods, func)

        return app.route(path, methods, func)

    return extender


import adacta.backend.web.api
import adacta.backend.web.frontend
