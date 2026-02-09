/* Built with Webpack 5 - depfused test lab */
import React, { useState, useEffect } from 'react';
import _ from 'lodash';
import { init as initApiClient } from '@xq9zk7823/api-client';
import { Card, Button } from '@xq9zk7823/design-system';

const apiClient = initApiClient({ baseUrl: 'https://dashboard-api.example.com' });

export const Dashboard = ({ userId }) => {
  const [stats, setStats] = useState(null);

  useEffect(() => {
    // Simulate loading dashboard stats
    const mockStats = {
      totalUsers: _.random(100, 10000),
      activeUsers: _.random(50, 5000),
      revenue: _.random(1000, 100000),
      growth: _.round(_.random(0.01, 0.50, true), 2),
    };
    setStats(mockStats);
    console.log('Dashboard initialized with API client:', apiClient);
  }, [userId]);

  if (!stats) {
    return React.createElement('div', null, 'Loading dashboard...');
  }

  return React.createElement('div', { className: 'dashboard' },
    React.createElement('h2', null, 'Dashboard'),
    React.createElement('div', { className: 'stats-grid' },
      Object.entries(stats).map(([key, value]) =>
        React.createElement(Card, { key, title: _.startCase(key) },
          React.createElement('p', { className: 'stat-value' }, String(value)),
          React.createElement(Button, { onClick: () => console.log('Drill into', key) }, 'Details')
        )
      )
    )
  );
};

export default Dashboard;
