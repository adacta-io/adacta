(function() {
  require.config({
    baseUrl: '/',
    paths: {
      'domReady': 'lib/requirejs-domready/js/domReady',
      'text': 'lib/requirejs-text/js/text',
      'jquery': 'lib/jquery/js/jquery',
      'bootstrap': 'lib/bootstrap/js/bootstrap.min',
      'knockout': 'lib/knockoutjs/js/knockout.debug',
      'pager': 'lib/pagerjs/js/pager.min',
      'pdfjs': 'lib/pdfjs-dist/js/pdf.combined',
      'typeahead': 'lib/typeahead.js/js/typeahead.bundle'
    },
    shim: {
      'bootstrap': {
        deps: ['jquery']
      },
      'bootstrap-tagsinput': {
        deps: ['bootstrap']
      }
    }
  });

  define(['knockout', 'pager', 'bootstrap', 'domReady!', 'pages/inbox', 'pages/robot', 'pages/archive', 'pages/settings', 'pages/user'], function(ko, pager) {
    var Site, site;
    Site = (function() {
      function Site() {
        this.inbox = require('pages/inbox');
        this.robot = require('pages/robot');
        this.archive = require('pages/archive');
        this.settings = require('pages/settings');
        this.user = require('pages/user');
      }


      /*
      Returns the pager.Page object with the given id.
       */

      Site.prototype.page = function(id) {
        return this.$__page__.child(id);
      };

      return Site;

    })();
    site = new Site;
    pager.extendWithPage(site);
    pager.onBindingError.add(console.error);
    pager.onSourceError.add(console.error);
    pager.start('inbox');
    ko.applyBindings(site);
    return site;
  });

}).call(this);
