define ['jquery', 'knockout', 'utils', 'api'], ($, ko, utils, api) ->
  class Document
    constructor: (@did) ->
      @thumbnailUrl = api().at('bundles').at(@did).at('thumbnail.png').path


  class Archive extends utils.RequestingModel
    constructor: () ->
      @querySearch = ko.observable ''

      @queryUploaded = [
        ko.observable null
        ko.observable null
      ]

      @queryTags = ko.observableArray []

      @query = ko.computed () =>
        queries = for tag in @queryTags()
          term:
            tags: tag

        if @querySearch()
          queries.push
            query_string:
              query: @querySearch()
              default_field: 'text'
              default_operator: 'AND'

        if @queryUploaded[0] or @queryUploaded[1]
          queries.push
            range:
              uploaded:
                gte: @queryUploaded[0]() || null
                lte: @queryUploaded[1]() || null

        query:
          bool:
            must: queries
        aggregations:
          tags:
            terms:
              field: 'tags'
              min_doc_count: 1

      @query.subscribe (q) -> console.log ko.toJSON q

      @documents = ko.observableArray []
      @tags = ko.observable null

      super
        request: ko.computed () => api().at('search').post(@query())
        process: (data) =>
          documents: (new Document hit._id for hit in data.hits.hits)
          tags: data.aggregations.tags.buckets
        data: (data) =>
          @documents data.documents
          @tags data.tags

      @count = ko.computed () =>
        return @documents().length


  return new Archive()
