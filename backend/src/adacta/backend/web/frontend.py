from adacta.backend.web import route

import bottle

import pkg_resources



#root = pkg_resources.resource_filename('adacta.backend.web.frontend', 'static')
root = 'frontend/build'


@route('/')
def index():
    return bottle.static_file('site.html',
                              root=root)


@route('/<path:path>')
def static(path):
    return bottle.static_file(path,
                              root=root)
