// Main entry point - re-exports from all modules
export { createApp, renderWidget } from './components.js';
export { formatAndLog, mergeConfigs, validateConfig } from './utils.js';

// Direct imports and re-exports from internal packages
import designSystem from '@xq9zk7823/design-system';
import i18nUtils from '@xq9zk7823/i18n-utils';
import { merge as lodashMerge } from 'lodash-es';

// Library initialization
export function initLibrary(options = {}) {
  const defaultOptions = {
    locale: 'en',
    theme: 'light',
    debug: false
  };

  const finalOptions = lodashMerge(defaultOptions, options);

  const ds = designSystem.init({ theme: finalOptions.theme });
  const i18n = i18nUtils.init({ locale: finalOptions.locale });

  return {
    designSystem: ds,
    i18n: i18n,
    options: finalOptions
  };
}

export const LIBRARY_VERSION = '1.0.0';
