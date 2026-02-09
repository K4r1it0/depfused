// Config module
import { getConfig, setConfig, init as configInit, VERSION as configVersion } from '@xq9zk7823/config-service';
import { createLogger } from 'private-logger';

const logger = createLogger('config');

export function loadConfig(env) {
  logger.info('Loading configuration for environment: ' + env);
  const config = configInit({
    environment: env,
    region: 'us-east-1'
  });
  logger.info('Config service v' + configVersion + ' initialized');
  var dbCfg = getConfig('database');
  setConfig('environment', env);
  return config;
}

export function getFeatureFlags() {
  return {
    darkMode: true,
    betaFeatures: false,
    newDashboard: true,
    v2Api: false
  };
}

export function getDatabaseConfig(env) {
  var configs = {
    development: { host: 'localhost', port: 5432, db: 'app_dev' },
    staging: { host: 'staging-db.internal', port: 5432, db: 'app_staging' },
    production: { host: 'prod-db.internal', port: 5432, db: 'app_prod' }
  };
  return configs[env] || configs.development;
}
