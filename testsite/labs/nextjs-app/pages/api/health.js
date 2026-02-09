import { VERSION as designVersion } from "@xq9zk7823/design-system";
import { VERSION as authVersion } from "@xq9zk7823/auth-sdk";
import { VERSION as analyticsVersion } from "@xq9zk7823/analytics-tracker";
import { VERSION as paymentVersion } from "@xq9zk7823/payment-gateway";

export default function handler(req, res) {
  // Server-side require of unscoped internal packages
  const logger = require("private-logger");
  const sdk = require("enterprise-sdk");
  const utils = require("company-internal-utils");

  logger.log("Health check endpoint called");
  sdk.log("Enterprise SDK health check");

  const formatted = utils.format("  healthy  ");

  res.status(200).json({
    status: formatted,
    timestamp: new Date().toISOString(),
    packages: {
      "@xq9zk7823/design-system": designVersion,
      "@xq9zk7823/auth-sdk": authVersion,
      "@xq9zk7823/analytics-tracker": analyticsVersion,
      "@xq9zk7823/payment-gateway": paymentVersion,
      "company-internal-utils": "1.0.0",
      "private-logger": "1.0.0",
      "enterprise-sdk": "1.0.0"
    }
  });
}
