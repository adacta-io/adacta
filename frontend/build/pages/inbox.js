(function() {
  var bind = function(fn, me){ return function(){ return fn.apply(me, arguments); }; },
    extend = function(child, parent) { for (var key in parent) { if (hasProp.call(parent, key)) child[key] = parent[key]; } function ctor() { this.constructor = child; } ctor.prototype = parent.prototype; child.prototype = new ctor(); child.__super__ = parent.prototype; return child; },
    hasProp = {}.hasOwnProperty;

  define(['jquery', 'knockout', 'utils', 'api'], function($, ko, utils, api) {
    var Document, Inbox;
    Document = (function(superClass) {
      extend(Document, superClass);

      function Document(did1) {
        this.did = did1;
        this.addTag = bind(this.addTag, this);
        this.removeTag = bind(this.removeTag, this);
        this.uploaded = ko.observable(null);
        this.tags = ko.observableArray([]);
        this.properties = ko.observable({});
        this.intermediateTag = ko.observable('');
        Document.__super__.constructor.call(this, {
          request: api().at('bundles').at(this.did).get(),
          data: (function(_this) {
            return function(data) {
              _this.uploaded(new Date(data.uploaded));
              _this.tags(data.tags);
              return _this.properties(data.properties);
            };
          })(this)
        });
        this.previewUrl = api().at('bundles').at(this.did).at('fragments').at('preview.png').path;
      }

      Document.prototype.removeTag = function(tag) {
        return this.tags.remove(tag);
      };

      Document.prototype.addTag = function() {
        this.tags.push(this.intermediateTag());
        this.tags.sort();
        return this.intermediateTag('');
      };

      return Document;

    })(utils.RequestingModel);
    Inbox = (function(superClass) {
      extend(Inbox, superClass);

      function Inbox() {
        this.archive = bind(this.archive, this);
        this.documents = ko.observableArray([]);
        Inbox.__super__.constructor.call(this, {
          request: api().at('search').at('inbox').get(),
          process: (function(_this) {
            return function(data) {
              var did, i, len, ref, results;
              ref = data.documents;
              results = [];
              for (i = 0, len = ref.length; i < len; i++) {
                did = ref[i];
                results.push(new Document(did));
              }
              return results;
            };
          })(this),
          data: this.documents
        });
        this.count = ko.computed((function(_this) {
          return function() {
            return _this.documents().length;
          };
        })(this));
      }

      Inbox.prototype.archive = function(document) {
        return this.ajax(api().at('bundles').at(document.did).put({
          reviewed: new Date(),
          tags: document.tags,
          properties: document.properties
        })).done((function(_this) {
          return function() {
            return _this.documents.remove(document);
          };
        })(this));
      };

      return Inbox;

    })(utils.RequestingModel);
    return new Inbox();
  });

}).call(this);
