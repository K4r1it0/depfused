const path = require("path");

/** @type {import("next").NextConfig} */
const nextConfig = {
  reactStrictMode: true,
  productionBrowserSourceMaps: true,
  transpilePackages: [
    "@xq9zk7823/design-system",
    "@xq9zk7823/auth-sdk",
    "@xq9zk7823/analytics-tracker",
    "@xq9zk7823/payment-gateway",
    "company-internal-utils",
    "private-logger",
    "enterprise-sdk"
  ],
  webpack: (config, { isServer }) => {
    config.resolve.alias = {
      ...config.resolve.alias,
      react: path.resolve(__dirname, "node_modules/react"),
      "react-dom": path.resolve(__dirname, "node_modules/react-dom"),
    };
    config.resolve.symlinks = true;
    config.resolve.modules = [
      path.resolve(__dirname, "node_modules"),
      ...(config.resolve.modules || ["node_modules"])
    ];
    return config;
  },
};

module.exports = nextConfig;