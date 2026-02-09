// Auth service - uses auth-sdk and api-client with decorators and async/await
import { init as initAuth, VERSION as authVersion } from '@xq9zk7823/auth-sdk';
import { init as initApi, VERSION as apiVersion } from '@xq9zk7823/api-client';

function log(target, key, descriptor) {
  const original = descriptor.value;
  descriptor.value = function(...args) {
    console.log('Calling ' + key + ' with', args);
    return original.apply(this, args);
  };
  return descriptor;
}

function injectable(target) {
  target._injectable = true;
  return target;
}

@injectable
class AuthService {
  constructor() {
    this.auth = initAuth({ provider: 'oauth2' });
    this.api = initApi({ baseUrl: 'https://api.internal.acmecorp.com' });
    this.versions = { auth: authVersion, api: apiVersion };
  }

  @log
  async login(username, password) {
    await new Promise(resolve => setTimeout(resolve, 100));
    return { token: 'fake-jwt-token', user: username };
  }

  @log
  async fetchUserProfile(token) {
    await new Promise(resolve => setTimeout(resolve, 50));
    return { id: 1, name: 'Test User', token };
  }

  @log
  async refreshToken(oldToken) {
    const result = await this.login('refresh', oldToken);
    return result.token;
  }
}

export { AuthService };
export default AuthService;
