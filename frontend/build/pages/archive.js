(function() {
  var extend = function(child, parent) { for (var key in parent) { if (hasProp.call(parent, key)) child[key] = parent[key]; } function ctor() { this.constructor = child; } ctor.prototype = parent.prototype; child.prototype = new ctor(); child.__super__ = parent.prototype; return child; },
    hasProp = {}.hasOwnProperty;

  define(['jquery', 'knockout', 'utils', 'api'], function($, ko, utils, api) {
    var Archive, Document;
    Document = (function() {
      function Document(did) {
        this.did = did;
        this.previewUrl = api().at('bundles').at(this.did).at('fragments').at('preview.png').path;
      }

      return Document;

    })();
    Archive = (function(superClass) {
      extend(Archive, superClass);

      function Archive() {
        this.querySearch = ko.observable('');
        this.queryUploaded = [ko.observable(null), ko.observable(null)];
        this.queryTags = ko.observableArray([]);
        this.query = ko.computed((function(_this) {
          return function() {
            var queries, tag;
            queries = (function() {
              var i, len, ref, results;
              ref = this.queryTags();
              results = [];
              for (i = 0, len = ref.length; i < len; i++) {
                tag = ref[i];
                results.push({
                  term: {
                    tags: tag
                  }
                });
              }
              return results;
            }).call(_this);
            if (_this.querySearch()) {
              queries.push({
                query_string: {
                  query: _this.querySearch(),
                  default_field: 'text',
                  default_operator: 'AND'
                }
              });
            }
            if (_this.queryUploaded[0] || _this.queryUploaded[1]) {
              queries.push({
                range: {
                  uploaded: {
                    gte: _this.queryUploaded[0]() || null,
                    lte: _this.queryUploaded[1]() || null
                  }
                }
              });
            }
            return {
              query: {
                bool: {
                  must: queries
                }
              },
              aggregations: {
                tags: {
                  terms: {
                    field: 'tags',
                    min_doc_count: 1
                  }
                }
              }
            };
          };
        })(this));
        this.query.subscribe(function(q) {
          return console.log(ko.toJSON(q));
        });
        this.documents = ko.observableArray([]);
        this.tags = ko.observable(null);
        Archive.__super__.constructor.call(this, {
          request: ko.computed((function(_this) {
            return function() {
              return api().at('search').post(_this.query());
            };
          })(this)),
          process: (function(_this) {
            return function(data) {
              var hit;
              return {
                documents: (function() {
                  var i, len, ref, results;
                  ref = data.hits.hits;
                  results = [];
                  for (i = 0, len = ref.length; i < len; i++) {
                    hit = ref[i];
                    results.push(new Document(hit._id));
                  }
                  return results;
                })(),
                tags: data.aggregations.tags.buckets
              };
            };
          })(this),
          data: (function(_this) {
            return function(data) {
              _this.documents(data.documents);
              return _this.tags(data.tags);
            };
          })(this)
        });
        this.count = ko.computed((function(_this) {
          return function() {
            return _this.documents().length;
          };
        })(this));
      }

      return Archive;

    })(utils.RequestingModel);
    return new Archive();
  });

}).call(this);
