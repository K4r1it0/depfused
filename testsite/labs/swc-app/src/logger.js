// Logger module - uses private-logger and lodash
import _ from 'lodash';
const privateLogger = require('private-logger');

export function createLogger(namespace) {
  return {
    info: (msg) => privateLogger.log('[INFO][' + namespace + '] ' + msg),
    error: (msg) => privateLogger.log('[ERROR][' + namespace + '] ' + msg),
    debug: (msg) => privateLogger.log('[DEBUG][' + namespace + '] ' + msg)
  };
}

export function formatMessage(template, data) {
  const formatted = privateLogger.format(template);
  return _.template(formatted)(data);
}

export function mergeLogConfigs(...configs) {
  return _.merge({}, ...configs);
}
