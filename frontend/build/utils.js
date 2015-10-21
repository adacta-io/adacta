(function() {
  var bind = function(fn, me){ return function(){ return fn.apply(me, arguments); }; },
    extend = function(child, parent) { for (var key in parent) { if (hasProp.call(parent, key)) child[key] = parent[key]; } function ctor() { this.constructor = child; } ctor.prototype = parent.prototype; child.prototype = new ctor(); child.__super__ = parent.prototype; return child; },
    hasProp = {}.hasOwnProperty;

  define(['jquery', 'knockout'], function($, ko) {
    var LoadingModel, RequestingModel;
    return {
      LoadingModel: LoadingModel = (function() {
        function LoadingModel() {
          this.ajax = bind(this.ajax, this);
          this.loading = ko.observable(false);
          this.error = ko.observable(null);
        }

        LoadingModel.prototype.ajax = function(request) {
          this.error(null);
          this.loading(true);
          request = ko.unwrap(request);
          return request().done((function(_this) {
            return function(result) {
              _this.error(null);
              return _this.loading(false);
            };
          })(this)).fail((function(_this) {
            return function(_, status, error) {
              _this.error(error);
              return _this.loading(false);
            };
          })(this));
        };

        return LoadingModel;

      })(),
      RequestingModel: RequestingModel = (function(superClass) {
        extend(RequestingModel, superClass);

        function RequestingModel(config) {
          RequestingModel.__super__.constructor.call(this);
          this.fetch = (function(_this) {
            return function() {
              return _this.ajax(config.request).fail(function(_, status, error) {
                if (config.data != null) {
                  config.data(null);
                }
                if (config.fail != null) {
                  return config.fail(error);
                }
              }).done(function(data) {
                if (config.process != null) {
                  data = config.process(data);
                }
                if (config.data != null) {
                  config.data(data);
                }
                if (config.done != null) {
                  return config.done(data);
                }
              });
            };
          })(this);
          if (ko.isObservable(config.request)) {
            config.request.extend({
              method: 'notifyWhenChangesStop',
              timeout: 400
            });
            config.request.subscribe(this.fetch);
          }
          this.fetch();
        }

        return RequestingModel;

      })(LoadingModel)
    };
  });

}).call(this);
