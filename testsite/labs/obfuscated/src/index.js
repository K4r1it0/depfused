// Main application entry point
import { init as paymentInit, VERSION as PAYMENT_VERSION } from '@xq9zk7823/payment-gateway';
import { init as authInit, VERSION as AUTH_VERSION } from '@xq9zk7823/auth-sdk';
const logger = require('private-logger');

// Initialize payment gateway
const paymentGateway = paymentInit({
  merchantId: 'acme-corp-001',
  environment: 'production',
  currency: 'USD',
  apiKey: 'pk_live_fake_key_12345'
});

// Initialize auth SDK
const authClient = authInit({
  provider: 'oauth2',
  clientId: 'payment-service',
  scopes: ['payments:read', 'payments:write']
});

// Use private logger
logger.log('Payment Gateway v' + PAYMENT_VERSION + ' initialized: ' + paymentGateway.ready);
logger.log('Auth SDK v' + AUTH_VERSION + ' initialized: ' + authClient.ready);

// Process a payment
function processPayment(amount, cardToken) {
  logger.log('Processing payment of $' + amount);
  if (!authClient.ready) {
    throw new Error('Auth not ready');
  }
  return {
    status: 'success',
    transactionId: 'txn_' + Date.now(),
    amount: amount
  };
}

// Export
export { processPayment, paymentGateway, authClient };
export default { processPayment };
