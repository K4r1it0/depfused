// Auth module
import { useAuth, init as authInit, VERSION } from '@xq9zk7823/auth-sdk';
import { createLogger } from 'private-logger';

const logger = createLogger('auth');

export function login(email, password) {
  logger.info('Attempting login for: ' + email);
  const authState = useAuth();
  authState.login();
  logger.info('Login completed');
  return { token: 'fake-jwt-token', email: email };
}

export function verifySession(token) {
  logger.debug('Verifying session token');
  return { valid: true, user: { id: 'user-123', role: 'admin' } };
}

export function getAuthHeaders(token) {
  return {
    'Authorization': 'Bearer ' + token,
    'X-Auth-Version': VERSION
  };
}
