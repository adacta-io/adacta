from require import *

import bottle


@export(app='adacta.backend.web:App')
def Api(app):
    # Generate a new bottle application containing the API
    api = bottle.Bottle()

    # Avoid fancy error pages for the API
    api.default_error_handler = lambda res: str(res.body)

    # Mount the API application to the root application
    app.mount('/api/v1',
              api)

    return api



def resource(path,
             methods='GET'):
    @require(api='adacta.backend.web.api:Api',
             logger='adacta.backend.utils:Logger')
    def extender(func,
                 api,
                 logger):
        logger.debug('Register API resource: %s %s -> %s',
                     path, methods, func)

        return api.route(path, methods, func)
    return extender



@export(oneshot)
def Request():
    return bottle.request



@export(oneshot)
def Response():
    return bottle.response



import adacta.backend.web.api.bundles
import adacta.backend.web.api.search
import adacta.backend.web.api.upload
