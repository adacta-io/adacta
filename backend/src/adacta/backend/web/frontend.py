from adacta.backend.web import route

import bottle



@route('/')
def index():
    return bottle.static_file('index.html',
                              'frontend/build')


@route('/<path:path>')
def static(path):
    return bottle.static_file(path,
                              'frontend/build')
