// Main entry point - esbuild Node.js app
// Static imports from fake internal packages
import { init as authInit, VERSION as authVersion } from "@xq9zk7823/auth-sdk";
import { init as configInit, VERSION as configVersion } from "@xq9zk7823/config-service";
import { createLogger, log, format } from "private-logger";
import { createClient, VERSION as sdkVersion } from "enterprise-sdk";

// Real package imports
import _ from "lodash";
import axios from "axios";

// Local module imports
import { login, verifySession, getAuthHeaders } from "./auth.js";
import { loadConfig, getFeatureFlags, getDatabaseConfig } from "./config.js";

// Initialize logger
const logger = createLogger("main");

// Initialize services
logger.info("Starting application...");
const authResult = authInit({ provider: "oauth2", clientId: "app-123" });
const configResult = configInit({ environment: "production" });
const client = createClient({ baseUrl: "https://api.enterprise.internal" });

logger.info("Auth SDK v" + authVersion + " initialized: " + JSON.stringify(authResult));
logger.info("Config Service v" + configVersion + " initialized: " + JSON.stringify(configResult));
logger.info("Enterprise SDK v" + sdkVersion + " initialized");

// Use lodash
const mergedConfig = _.merge({}, authResult, configResult, {
  features: getFeatureFlags(),
  database: getDatabaseConfig("production"),
  timestamp: _.now()
});

log(format("  Application configuration loaded  "));
logger.info("Merged config: " + JSON.stringify(_.pick(mergedConfig, ["ready", "features"])));

// Use axios
const apiClient = axios.create({
  baseURL: "https://api.example.com",
  timeout: 5000,
  headers: getAuthHeaders("fake-token-123")
});

// Dynamic import of internal package
async function loadDynamicModules() {
  const dynamicAuth = await import("@xq9zk7823/auth-sdk");
  logger.info("Dynamically loaded auth-sdk v" + dynamicAuth.VERSION);
  
  const dynamicConfig = await import("@xq9zk7823/config-service");
  logger.info("Dynamically loaded config-service v" + dynamicConfig.VERSION);
  
  return { dynamicAuth, dynamicConfig };
}

// Application logic
async function main() {
  try {
    const loginResult = login("admin@xq9zk7823.com", "secure-password");
    logger.info("Login result: " + JSON.stringify(loginResult));

    const session = verifySession(loginResult.token);
    logger.info("Session verified: " + JSON.stringify(session));

    const appConfig = loadConfig("production");
    logger.info("App config loaded: " + JSON.stringify(appConfig));

    const response = await client.get("/api/v1/status");
    logger.info("API status: " + JSON.stringify(response));

    const dynamicModules = await loadDynamicModules();
    logger.info("Dynamic modules loaded successfully");

    const summary = {
      loginSuccess: _.get(loginResult, "token", null) !== null,
      sessionValid: _.get(session, "valid", false),
      configReady: _.get(appConfig, "ready", false),
      apiStatus: _.get(response, "status", 0),
      lodashVersion: _.VERSION,
      axiosVersion: axios.VERSION
    };

    logger.info("Application summary: " + JSON.stringify(summary));
    return summary;
  } catch (error) {
    logger.error("Application error: " + error.message);
    throw error;
  }
}

export { main, mergedConfig, apiClient, loadDynamicModules };
export default main;

main().then(function(result) {
  logger.info("Application completed successfully");
}).catch(function(err) {
  logger.error("Application failed: " + err.message);
});