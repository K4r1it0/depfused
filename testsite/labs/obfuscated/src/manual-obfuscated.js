// Manual obfuscation patterns for testing scanner detection capabilities

// Pattern 1: Base64 encoded requires
// "@xq9zk7823/auth-sdk" = "QGFjbWVjb3JwL2F1dGgtc2Rr"
const authSdk = require(atob("QGFjbWVjb3JwL2F1dGgtc2Rr"));

// Pattern 2: Base64 encoded package name for @xq9zk7823/payment-gateway
// "@xq9zk7823/payment-gateway" = "QGFjbWVjb3JwL3BheW1lbnQtZ2F0ZXdheQ=="
const paymentGw = require(atob("QGFjbWVjb3JwL3BheW1lbnQtZ2F0ZXdheQ=="));

// Pattern 3: Hex encoded string for 'private-logger'
const _0x1a2b = '\x70\x72\x69\x76\x61\x74\x65\x2d\x6c\x6f\x67\x67\x65\x72';
const logger1 = require(_0x1a2b);

// Pattern 4: String.fromCharCode for '@xq9zk7823/auth-sdk'
const _pkg1 = String.fromCharCode(64,97,99,109,101,99,111,114,112,47,97,117,116,104,45,115,100,107);
const authSdk2 = require(_pkg1);

// Pattern 5: Array join for '@xq9zk7823/payment-gateway'
const _parts = ['@acme','corp','/pay','ment-','gate','way'];
const paymentGw2 = require(_parts.join(''));

// Pattern 6: String concatenation
const _prefix = '@xq9zk7823/';
const _name = 'auth' + '-' + 'sdk';
const authSdk3 = require(_prefix + _name);

// Pattern 7: Reverse string
function _rev(s) { return s.split('').reverse().join(''); }
const _reversed = _rev('kds-htua/procemca@');
const authSdk4 = require(_reversed);

// Pattern 8: Variable indirection with hex property names
const _0xabc = {};
_0xabc['\x6e\x61\x6d\x65'] = '\x70\x72\x69\x76\x61\x74\x65\x2d\x6c\x6f\x67\x67\x65\x72';
const logger2 = require(_0xabc['\x6e\x61\x6d\x65']);

// Pattern 9: Template literal construction
const scope = 'acmecorp';
const pkg = 'payment-gateway';
const logger3 = require(`@${scope}/${pkg}`);

// Pattern 10: Buffer decode (Node.js specific)
const _b64 = Buffer.from('QGFjbWVjb3JwL2F1dGgtc2Rr', 'base64').toString();
const authSdk5 = require(_b64);

// Use all the imported modules
authSdk.init({ mode: 'test' });
paymentGw.init({ merchant: 'test' });
logger1.log('Manual obfuscation patterns loaded');
logger2.log('All patterns working');

module.exports = { authSdk, paymentGw, logger1 };
