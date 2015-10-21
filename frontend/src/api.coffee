define ['jquery', 'knockout'], ($, ko) ->
  class Resource
    constructor: (@path='/api/v1') ->


    at: (path) =>
      new Resource("#{@path}/#{path}")


    using: (params) =>
      new Resource("#{@path}?#{$.param ko.toJS params}")


    get: () =>
      () =>
        console.log 'GET', @path
        $.ajax @path,
          type: 'GET'
          accepts: 'application/json'


    put: (data) =>
      () =>
        $.ajax @path,
          type: 'PUT'
          contentType: 'application/json'
          accepts: 'application/json'
          data: ko.toJSON data


    post: (data) =>
      () =>
        $.ajax @path,
          type: 'POST'
          contentType: 'application/json'
          accepts: 'application/json'
          data: ko.toJSON data


    delete: () =>
      () =>
        console.log 'DELETE', @path
        $.ajax @path,
          type: 'DELETE'
          accepts: 'application/json'



  return () ->
    new Resource()
