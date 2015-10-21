define ['jquery', 'knockout'], ($, ko) ->

  # A helper base class for models providing loading and error support.
  #
  # The class provides a 'loading' observable indicating the current state and
  # an 'error' observable containing the occurred error, if any.
  #
  # To wrap AJAX request in a loading block, the 'ajax' method can be used. It
  # takes a request executor function which executes the request call. The
  # function is called and must return the promise of that request.
  LoadingModel: class LoadingModel
    constructor: () ->
      @loading = ko.observable false
      @error = ko.observable null


    ajax: (request) =>
      # Reset state to loading
      @error null
      @loading true

      # Get the request executor
      request = ko.unwrap request

      # Execute the request
      request()
      .done (result) =>
        # Set state to done successfully
        @error null
        @loading false
      .fail (_, status, error) =>
        # Set state to error
        @error error
        @loading false



  # A helper base class for models doing simple AJAX requests.
  #
  # All extenders to this class must provide a request field containing a
  # request function as described in LoadingModel. This can be an observable.
  #
  # The 'fetch' method can be used to trigger an AJAX request using the provided request.
  #
  # In addition, if the 'request' is an observable and it changes, the request
  # is triggered automatically (using a rate limitation waiting for the changes
  # to settle). The result of the AJAX request is stored passed to the process
  # function, id such a function exists. The outcome of that function is stored
  # in the 'data' observable and is passed to the 'done' function if such a
  # function exists. Errors will clean the 'data' observable and will trigger
  # the 'fail' function if such a function exists.
  #
  # TODO: Instead of calling the done / fail function, some kind of deferred chaining should be used.
  RequestingModel: class RequestingModel extends LoadingModel
    constructor: (config) ->
      super()

      @fetch = () =>
        # Execute the request
        @ajax config.request
        .fail (_, status, error) =>
          if config.data? then config.data null
          if config.fail? then config.fail error
        .done (data) =>
          if config.process? then data = config.process data
          if config.data? then config.data data
          if config.done? then config.done data

      # Configure rate limiting on the request observable and subscribe to execute the request on changes
      if ko.isObservable config.request
        config.request.extend
          method: 'notifyWhenChangesStop'
          timeout: 400
        config.request.subscribe @fetch

      # Execute the request initially
      @fetch()
