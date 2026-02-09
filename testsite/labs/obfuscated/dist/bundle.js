var __create = Object.create;
var __defProp = Object.defineProperty;
var __getOwnPropDesc = Object.getOwnPropertyDescriptor;
var __getOwnPropNames = Object.getOwnPropertyNames;
var __getProtoOf = Object.getPrototypeOf;
var __hasOwnProp = Object.prototype.hasOwnProperty;
var __commonJS = (cb, mod) => function __require() {
  return mod || (0, cb[__getOwnPropNames(cb)[0]])((mod = { exports: {} }).exports, mod), mod.exports;
};
var __export = (target, all) => {
  for (var name in all)
    __defProp(target, name, { get: all[name], enumerable: true });
};
var __copyProps = (to, from, except, desc) => {
  if (from && typeof from === "object" || typeof from === "function") {
    for (let key of __getOwnPropNames(from))
      if (!__hasOwnProp.call(to, key) && key !== except)
        __defProp(to, key, { get: () => from[key], enumerable: !(desc = __getOwnPropDesc(from, key)) || desc.enumerable });
  }
  return to;
};
var __toESM = (mod, isNodeMode, target) => (target = mod != null ? __create(__getProtoOf(mod)) : {}, __copyProps(
  // If the importer is in node compatibility mode or this is not an ESM
  // file that has been converted to a CommonJS file using a Babel-
  // compatible transform (i.e. "__esModule" has not been set), then set
  // "default" to the CommonJS "module.exports" for node compatibility.
  isNodeMode || !mod || !mod.__esModule ? __defProp(target, "default", { value: mod, enumerable: true }) : target,
  mod
));
var __toCommonJS = (mod) => __copyProps(__defProp({}, "__esModule", { value: true }), mod);

// node_modules/@xq9zk7823/payment-gateway/index.js
var require_payment_gateway = __commonJS({
  "node_modules/@xq9zk7823/payment-gateway/index.js"(exports2) {
    "use strict";
    var createPayment = function createPayment2(amount, currency) {
      return { id: "pay_" + Math.random().toString(36).substr(2, 9), amount, currency, status: "pending" };
    };
    var processPayment2 = function processPayment3(id) {
      return { id, status: "completed", timestamp: Date.now() };
    };
    var refundPayment = function refundPayment2(id, amount) {
      return { id, refundAmount: amount, status: "refunded" };
    };
    function PaymentClient(config) {
      this.apiKey = config.apiKey;
    }
    PaymentClient.prototype.charge = function(amount) {
      return createPayment(amount, "USD");
    };
    var init = function(config) {
      return { ready: true };
    };
    var VERSION = "1.0.0";
    exports2.createPayment = createPayment;
    exports2.processPayment = processPayment2;
    exports2.refundPayment = refundPayment;
    exports2.PaymentClient = PaymentClient;
    exports2.init = init;
    exports2.VERSION = VERSION;
    exports2.default = { createPayment, processPayment: processPayment2, refundPayment, PaymentClient, init, VERSION };
  }
});

// node_modules/@xq9zk7823/auth-sdk/index.js
var require_auth_sdk = __commonJS({
  "node_modules/@xq9zk7823/auth-sdk/index.js"(exports2) {
    "use strict";
    function AuthProvider(props) {
      return "<div data-auth=provider>" + (props && props.children || "") + "</div>";
    }
    function LoginForm() {
      return "<form class=acme-login><input type=email placeholder=Email /><button type=submit>Login</button></form>";
    }
    function useAuth() {
      return { user: null, isAuthenticated: false, login: function() {
      }, logout: function() {
      } };
    }
    var init = function(config) {
      return { ready: true };
    };
    var VERSION = "1.0.0";
    exports2.AuthProvider = AuthProvider;
    exports2.LoginForm = LoginForm;
    exports2.useAuth = useAuth;
    exports2.init = init;
    exports2.VERSION = VERSION;
  }
});

// node_modules/private-logger/index.js
var require_private_logger = __commonJS({
  "node_modules/private-logger/index.js"(exports2) {
    var createLogger = function(namespace) {
      return {
        info: function(msg) {
          console.log("[INFO][" + namespace + "] " + msg);
        },
        warn: function(msg) {
          console.warn("[WARN][" + namespace + "] " + msg);
        },
        error: function(msg) {
          console.error("[ERROR][" + namespace + "] " + msg);
        },
        debug: function(msg) {
          console.log("[DEBUG][" + namespace + "] " + msg);
        }
      };
    };
    var log = function(msg) {
      console.log(msg);
    };
    var format = function(s) {
      return s.trim();
    };
    exports2.createLogger = createLogger;
    exports2.log = log;
    exports2.format = format;
  }
});

// src/index.js
var src_exports = {};
__export(src_exports, {
  authClient: () => authClient,
  default: () => src_default,
  paymentGateway: () => paymentGateway,
  processPayment: () => processPayment
});
module.exports = __toCommonJS(src_exports);
var import_payment_gateway = __toESM(require_payment_gateway());
var import_auth_sdk = __toESM(require_auth_sdk());
var logger = require_private_logger();
var paymentGateway = (0, import_payment_gateway.init)({
  merchantId: "acme-corp-001",
  environment: "production",
  currency: "USD",
  apiKey: "pk_live_fake_key_12345"
});
var authClient = (0, import_auth_sdk.init)({
  provider: "oauth2",
  clientId: "payment-service",
  scopes: ["payments:read", "payments:write"]
});
logger.log("Payment Gateway v" + import_payment_gateway.VERSION + " initialized: " + paymentGateway.ready);
logger.log("Auth SDK v" + import_auth_sdk.VERSION + " initialized: " + authClient.ready);
function processPayment(amount, cardToken) {
  logger.log("Processing payment of $" + amount);
  if (!authClient.ready) {
    throw new Error("Auth not ready");
  }
  return {
    status: "success",
    transactionId: "txn_" + Date.now(),
    amount
  };
}
var src_default = { processPayment };
// Annotate the CommonJS export names for ESM import in node:
0 && (module.exports = {
  authClient,
  paymentGateway,
  processPayment
});
