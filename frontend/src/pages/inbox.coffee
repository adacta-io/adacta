define ['jquery', 'knockout', 'utils', 'api'], ($, ko, utils, api) ->
  class Document extends utils.RequestingModel
    constructor: (@did) ->
      @uploaded = ko.observable null
      @tags = ko.observableArray []
      @properties = ko.observable {}

      @intermediateTag = ko.observable ''

      super
        request: api().at('bundles').at(@did).get()
        data: (data) =>
          @uploaded new Date data.uploaded
          @tags data.tags
          @properties data.properties

       @previewUrl = api().at('bundles').at(@did).at('fragments').at('preview.png').path


    removeTag: (tag) =>
      @tags.remove tag


    addTag: () =>
      @tags.push @intermediateTag()
      @tags.sort()

      @intermediateTag ''



  class Inbox extends utils.RequestingModel
    constructor: () ->
      @documents = ko.observableArray []

      super
        request: api().at('search').at('inbox').get()
        process: (data) => new Document did for did in data.documents
        data: @documents

      @count = ko.computed () =>
        return @documents().length


    archive: (document) =>
      @ajax api().at('bundles').at(document.did).put
        reviewed: new Date()
        tags: document.tags
        properties: document.properties
      .done () =>
        @documents.remove document



  return new Inbox()
