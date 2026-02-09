/* Built with Webpack 5 - depfused test lab */
import React from 'react';
import { VERSION, Header as DsHeader } from '@xq9zk7823/design-system';

export const AppHeader = ({ title }) => {
  return React.createElement('div', { className: 'app-header-wrapper' },
    React.createElement(DsHeader, { title: title }),
    React.createElement('span', { className: 'ds-version' }, 'DS v' + VERSION)
  );
};
