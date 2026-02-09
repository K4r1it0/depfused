"use strict";

var createPayment = function createPayment(amount, currency) { return { id: "pay_" + Math.random().toString(36).substr(2, 9), amount: amount, currency: currency, status: "pending" }; };
var processPayment = function processPayment(id) { return { id: id, status: "completed", timestamp: Date.now() }; };
var refundPayment = function refundPayment(id, amount) { return { id: id, refundAmount: amount, status: "refunded" }; };
function PaymentClient(config) { this.apiKey = config.apiKey; }
PaymentClient.prototype.charge = function(amount) { return createPayment(amount, "USD"); };
var init = function(config) { return { ready: true }; };
var VERSION = "1.0.0";

exports.createPayment = createPayment;
exports.processPayment = processPayment;
exports.refundPayment = refundPayment;
exports.PaymentClient = PaymentClient;
exports.init = init;
exports.VERSION = VERSION;
exports.default = { createPayment: createPayment, processPayment: processPayment, refundPayment: refundPayment, PaymentClient: PaymentClient, init: init, VERSION: VERSION };