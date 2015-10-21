(function() {
  var bind = function(fn, me){ return function(){ return fn.apply(me, arguments); }; };

  define(['jquery', 'knockout'], function($, ko) {
    var Resource;
    Resource = (function() {
      function Resource(path1) {
        this.path = path1 != null ? path1 : '/api/v1';
        this["delete"] = bind(this["delete"], this);
        this.post = bind(this.post, this);
        this.put = bind(this.put, this);
        this.get = bind(this.get, this);
        this.using = bind(this.using, this);
        this.at = bind(this.at, this);
      }

      Resource.prototype.at = function(path) {
        return new Resource(this.path + "/" + path);
      };

      Resource.prototype.using = function(params) {
        return new Resource(this.path + "?" + ($.param(ko.toJS(params))));
      };

      Resource.prototype.get = function() {
        return (function(_this) {
          return function() {
            console.log('GET', _this.path);
            return $.ajax(_this.path, {
              type: 'GET',
              accepts: 'application/json'
            });
          };
        })(this);
      };

      Resource.prototype.put = function(data) {
        return (function(_this) {
          return function() {
            return $.ajax(_this.path, {
              type: 'PUT',
              contentType: 'application/json',
              accepts: 'application/json',
              data: ko.toJSON(data)
            });
          };
        })(this);
      };

      Resource.prototype.post = function(data) {
        return (function(_this) {
          return function() {
            return $.ajax(_this.path, {
              type: 'POST',
              contentType: 'application/json',
              accepts: 'application/json',
              data: ko.toJSON(data)
            });
          };
        })(this);
      };

      Resource.prototype["delete"] = function() {
        return (function(_this) {
          return function() {
            console.log('DELETE', _this.path);
            return $.ajax(_this.path, {
              type: 'DELETE',
              accepts: 'application/json'
            });
          };
        })(this);
      };

      return Resource;

    })();
    return function() {
      return new Resource();
    };
  });

}).call(this);
