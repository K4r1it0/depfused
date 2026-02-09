// Utils module - uses CJS require for company-internal-utils and lodash-es
import { merge, cloneDeep, isEmpty } from 'lodash-es';

// CJS-style require for the unscoped internal package
const internalUtils = require('company-internal-utils');

export function formatAndLog(message) {
  const formatted = internalUtils.format(message);
  internalUtils.log(formatted);
  return formatted;
}

export function mergeConfigs(base, override) {
  return merge(cloneDeep(base), override);
}

export function validateConfig(config) {
  if (isEmpty(config)) {
    formatAndLog('Warning: empty config provided');
    return false;
  }
  formatAndLog('Config validated successfully');
  return true;
}
