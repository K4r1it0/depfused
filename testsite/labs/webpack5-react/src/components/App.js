/* Built with Webpack 5 - depfused test lab */
import React, { useState, useEffect, useCallback } from 'react';
import _ from 'lodash';
import { Button, Card } from '@xq9zk7823/design-system';

const App = ({ config, header, footer, onLoad }) => {
  const [loaded, setLoaded] = useState(false);
  const [items, setItems] = useState([]);

  const processItems = useCallback((rawItems) => {
    return _.chain(rawItems)
      .filter(item => item.active)
      .sortBy('name')
      .map(item => ({ ...item, processed: true }))
      .value();
  }, []);

  useEffect(() => {
    if (!loaded) {
      setLoaded(true);
      if (onLoad) onLoad();

      // Simulate data
      const rawItems = [
        { id: 1, name: 'Widget A', active: true },
        { id: 2, name: 'Widget B', active: false },
        { id: 3, name: 'Widget C', active: true },
        { id: 4, name: 'Gadget D', active: true },
      ];
      setItems(processItems(rawItems));
    }
  }, [loaded, onLoad, processItems]);

  return React.createElement('div', { className: 'app-container' },
    header,
    React.createElement('main', null,
      React.createElement('h2', null, 'Application: ' + config.name),
      React.createElement('p', null, 'Version: ' + config.version),
      React.createElement('p', null, 'Environment: ' + config.env),
      React.createElement('div', { className: 'card-list' },
        items.map(item =>
          React.createElement(Card, { key: item.id, title: item.name },
            React.createElement('p', null, item.processed ? 'Processed' : 'Pending'),
            React.createElement(Button, { onClick: () => console.log('clicked', item.id) }, 'View ' + item.name)
          )
        )
      )
    ),
    footer
  );
};

export default App;
