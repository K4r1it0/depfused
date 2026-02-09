// src/manual-obfuscated.js
var authSdk = require(atob("QGFjbWVjb3JwL2F1dGgtc2Rr"));
var paymentGw = require(atob("QGFjbWVjb3JwL3BheW1lbnQtZ2F0ZXdheQ=="));
var _0x1a2b = "private-logger";
var logger1 = require(_0x1a2b);
var _pkg1 = String.fromCharCode(64, 97, 99, 109, 101, 99, 111, 114, 112, 47, 97, 117, 116, 104, 45, 115, 100, 107);
var authSdk2 = require(_pkg1);
var _parts = ["@acme", "corp", "/pay", "ment-", "gate", "way"];
var paymentGw2 = require(_parts.join(""));
var _prefix = "@xq9zk7823/";
var _name = "auth-sdk";
var authSdk3 = require(_prefix + _name);
function _rev(s) {
  return s.split("").reverse().join("");
}
var _reversed = _rev("kds-htua/procemca@");
var authSdk4 = require(_reversed);
var _0xabc = {};
_0xabc["name"] = "private-logger";
var logger2 = require(_0xabc["name"]);
var scope = "acmecorp";
var pkg = "payment-gateway";
var logger3 = require(`@${scope}/${pkg}`);
var _b64 = Buffer.from("QGFjbWVjb3JwL2F1dGgtc2Rr", "base64").toString();
var authSdk5 = require(_b64);
authSdk.init({ mode: "test" });
paymentGw.init({ merchant: "test" });
logger1.log("Manual obfuscation patterns loaded");
logger2.log("All patterns working");
module.exports = { authSdk, paymentGw, logger1 };
