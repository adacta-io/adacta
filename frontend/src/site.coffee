require.config
  baseUrl: '/'
  paths:
    'domReady': 'lib/requirejs-domready/js/domReady'
    'text': 'lib/requirejs-text/js/text'
    'jquery': 'lib/jquery/js/jquery'
    'bootstrap': 'lib/bootstrap/js/bootstrap.min'
    'knockout': 'lib/knockoutjs/js/knockout.debug'
    'pager': 'lib/pagerjs/js/pager.min'
    'pdfjs': 'lib/pdfjs-dist/js/pdf.combined'
    'typeahead': 'lib/typeahead.js/js/typeahead.bundle'
  shim:
    'bootstrap':
      deps: [ 'jquery' ]
    'bootstrap-tagsinput':
      deps: ['bootstrap']


define ['knockout', 'pager', 'bootstrap', 'domReady!', 'pages/inbox', 'pages/robot', 'pages/archive', 'pages/settings', 'pages/user'], (ko, pager) ->

  class Site
    constructor: () ->
      @inbox = require('pages/inbox')
      @robot = require('pages/robot')
      @archive = require('pages/archive')
      @settings = require('pages/settings')
      @user = require('pages/user')


    ###
    Returns the pager.Page object with the given id.
    ###
    page: (id) ->
      this.$__page__.child(id)


  # Create the main VM instance
  site = new Site

  # Extend the main VM instance with pagerjs structures
  pager.extendWithPage site

  # Enable error logging for pagerjs
  pager.onBindingError.add console.error
  pager.onSourceError.add console.error

  # Start routing and navigation
  pager.start('inbox')

  # Bind the main VM to the DOM
  ko.applyBindings site

  # Return the main VM instance for use in other modules
  return site
