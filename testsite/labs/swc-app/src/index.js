// Main entry point - combines auth service and logger
import { AuthService } from './auth-service.js';
import { createLogger, formatMessage, mergeLogConfigs } from './logger.js';
import * as authSdk from '@xq9zk7823/auth-sdk';
import * as apiClient from '@xq9zk7823/api-client';
import _ from 'lodash';

// Re-export everything
export { AuthService } from './auth-service.js';
export { createLogger, formatMessage, mergeLogConfigs } from './logger.js';
export * from '@xq9zk7823/auth-sdk';
export * from '@xq9zk7823/api-client';

// App initialization with async
export async function initApp(config = {}) {
  const logger = createLogger('app');
  logger.info('Initializing application...');

  const defaultConfig = {
    auth: { provider: 'oauth2' },
    api: { baseUrl: 'https://api.acmecorp.com' },
    logging: { level: 'info' }
  };

  const finalConfig = _.merge(defaultConfig, config);
  
  const authService = new AuthService();
  
  logger.info('App initialized with config: ' + JSON.stringify(finalConfig));

  return {
    authService,
    config: finalConfig,
    logger,
    sdkVersions: {
      auth: authSdk.VERSION,
      api: apiClient.VERSION
    }
  };
}

export const APP_VERSION = '1.0.0';
