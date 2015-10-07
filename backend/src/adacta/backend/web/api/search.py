from require import *
from adacta.backend.web.api import resource



@resource('/search', ['GET'])
@require(index='adacta.backend.index:Index',
         request='adacta.backend.web.api:Request')
def search(index,
           request):
    query = request.params.getone('q', default='*')
    tags = request.params.getall('tag')

    return index.search(query=query,
                        tags=tags)
