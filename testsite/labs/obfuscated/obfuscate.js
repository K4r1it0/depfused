const JavaScriptObfuscator = require('javascript-obfuscator');
const fs = require('fs');
const path = require('path');

const bundleCode = fs.readFileSync(path.join(__dirname, 'dist/bundle.js'), 'utf8');

// Low obfuscation
const lowResult = JavaScriptObfuscator.obfuscate(bundleCode, {
  compact: true,
  stringArray: true,
  stringArrayThreshold: 0.75
});
fs.writeFileSync(path.join(__dirname, 'dist/obfuscated-low.js'), lowResult.getObfuscatedCode());
console.log('Created dist/obfuscated-low.js');

// Medium obfuscation
const medResult = JavaScriptObfuscator.obfuscate(bundleCode, {
  compact: true,
  stringArray: true,
  stringArrayEncoding: ['base64'],
  stringArrayThreshold: 0.75,
  renameGlobals: false
});
fs.writeFileSync(path.join(__dirname, 'dist/obfuscated-medium.js'), medResult.getObfuscatedCode());
console.log('Created dist/obfuscated-medium.js');

// High obfuscation
const highResult = JavaScriptObfuscator.obfuscate(bundleCode, {
  compact: true,
  stringArray: true,
  stringArrayEncoding: ['rc4'],
  stringArrayThreshold: 1,
  selfDefending: true,
  renameGlobals: false,
  deadCodeInjection: true,
  deadCodeInjectionThreshold: 0.4
});
fs.writeFileSync(path.join(__dirname, 'dist/obfuscated-high.js'), highResult.getObfuscatedCode());
console.log('Created dist/obfuscated-high.js');

console.log('All obfuscation levels complete!');
