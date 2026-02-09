// private-logger internal package stub
var createLogger = function(namespace) {
  return {
    info: function(msg) { console.log('[INFO][' + namespace + '] ' + msg); },
    warn: function(msg) { console.warn('[WARN][' + namespace + '] ' + msg); },
    error: function(msg) { console.error('[ERROR][' + namespace + '] ' + msg); },
    debug: function(msg) { console.log('[DEBUG][' + namespace + '] ' + msg); }
  };
};
var log = function(msg) { console.log(msg); };
var format = function(s) { return s.trim(); };
exports.createLogger = createLogger;
exports.log = log;
exports.format = format;
