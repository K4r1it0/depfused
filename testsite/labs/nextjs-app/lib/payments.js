// Payment utilities using internal packages
const { PaymentClient, createPayment } = require("@xq9zk7823/payment-gateway");
const { log } = require("enterprise-sdk");

function getPaymentClient() {
  log("Creating payment client");
  return new PaymentClient({ apiKey: "test_key_123" });
}

function quickPay(amount, currency) {
  log("Quick pay: " + amount + " " + currency);
  return createPayment(amount, currency || "USD");
}

module.exports = { getPaymentClient, quickPay };
