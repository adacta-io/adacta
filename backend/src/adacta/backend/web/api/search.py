from require import *
from adacta.backend.web.api import resource



# @resource('/search', ['GET'])
# @require(index='adacta.backend.index:Index',
#          request='adacta.backend.web.api:Request')
# def search(index,
#            request):
#     body = {
#     }
#
#     query = request.params.getone('q', default=None)
#     if query:
#         body['query'] = {
#             'simple_query_string': {
#                 'query': query,
#             }
#         }
#
#     tags = request.params.getall('tag')
#     if tags:
#         body['filter'] = {
#             'and': [
#                 {
#                     'term': {
#                         'tags': tag
#                     }
#                 } for tag in tags]
#         }
#
#     return index.search(body)



@resource('/search', ['POST'])
@require(index='adacta.backend.index:Index',
         request='adacta.backend.web.api:Request')
def search(index,
           request):
    return index.search(request.json)



@resource('/search/inbox', ['GET'])
@require(index='adacta.backend.index:Index')
def inbox(index):
    result = index.search(filter={
        'not': {
            'exists': {
                'field': 'reviewed'
            }
        }
    })

    return {
        'documents': [hit['_id']
                      for hit
                      in result['hits']['hits']]
    }
