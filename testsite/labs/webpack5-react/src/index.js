/* Built with Webpack 5 - depfused test lab */
import React from 'react';
import { createRoot } from 'react-dom/client';
import _ from 'lodash';

// Static import of internal design system package (CJS - webpack handles interop)
import { Button, Card, Header as DsHeader, Footer as DsFooter, init as initDesignSystem, VERSION as dsVersion } from '@xq9zk7823/design-system';

// CommonJS require of internal auth SDK
const authSdk = require('@xq9zk7823/auth-sdk');

// Static import of internal packages (ESM)
import { init as initConfig } from '@xq9zk7823/config-service';
import { init as initApiClient } from '@xq9zk7823/api-client';

// Unscoped internal packages (CJS)
const internalUtils = require('company-internal-utils');
const privateLogger = require('private-logger');

// Components
import App from './components/App';
import { AppHeader } from './components/Header';
import { AppFooter } from './components/Footer';

// Services
import { apiService } from './services/api';

// Initialize internal packages
const dsResult = initDesignSystem({ theme: 'dark' });
const authResult = authSdk.init({ clientId: 'test-app' });
const configResult = initConfig({ env: 'production' });
const apiClientResult = initApiClient({ baseUrl: 'https://api.example.com' });

// Use lodash to demonstrate real vendor dependency
const appConfig = _.merge(
  { name: 'webpack5-react-testlab' },
  { version: '1.0.0', env: 'production' }
);

// Use unscoped internal packages
internalUtils.log('App starting with config: ' + JSON.stringify(appConfig));
privateLogger.log('Design system version: ' + dsVersion);

// Log init results
console.log('Design System:', dsResult);
console.log('Auth SDK:', authResult);
console.log('Config:', configResult);
console.log('API Client:', apiClientResult);

// Use design system components
console.log('Button component:', Button);
console.log('Card component:', Card);

// Dynamic import for code splitting - analytics tracker
async function loadAnalytics() {
  const analytics = await import('@xq9zk7823/analytics-tracker');
  analytics.init({ trackingId: 'UA-TEST-1' });
  analytics.trackEvent('app_loaded', { timestamp: Date.now() });
  return analytics;
}

// Another dynamic import for code splitting - dashboard component
async function loadDashboard() {
  const { Dashboard } = await import('./components/Dashboard');
  return Dashboard;
}

// Mount the React app
const container = document.getElementById('root');
const root = createRoot(container);

root.render(
  React.createElement(App, {
    config: appConfig,
    header: React.createElement(AppHeader, { title: appConfig.name }),
    footer: React.createElement(AppFooter, { version: appConfig.version }),
    onLoad: () => {
      loadAnalytics().then(a => console.log('Analytics loaded:', a));
      loadDashboard().then(D => console.log('Dashboard component loaded:', D));
    }
  })
);

// Use the api service
apiService.fetchData('/users').then(data => console.log('Users:', data));
