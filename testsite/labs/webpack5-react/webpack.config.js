/* Built with Webpack 5 - depfused test lab */
const path = require('path');
const HtmlWebpackPlugin = require('html-webpack-plugin');

module.exports = {
  mode: 'production',

  entry: {
    main: './src/index.js',
  },

  output: {
    path: path.resolve(__dirname, 'dist'),
    filename: '[name].[contenthash:8].js',
    chunkFilename: '[name].[contenthash:8].chunk.js',
    clean: true,
  },

  devtool: 'source-map',

  module: {
    rules: [
      {
        test: /\.(js|jsx)$/,
        exclude: /node_modules/,
        use: {
          loader: 'babel-loader',
          options: {
            presets: [
              ['@babel/preset-env', { targets: '> 0.25%, not dead' }],
              ['@babel/preset-react', { runtime: 'automatic' }],
            ],
          },
        },
      },
      {
        test: /\.css$/,
        use: ['style-loader', 'css-loader'],
      },
    ],
  },

  resolve: {
    extensions: ['.js', '.jsx'],
    // Ensure linked packages (file: protocol) can resolve shared deps from project node_modules
    modules: [path.resolve(__dirname, 'node_modules'), 'node_modules'],
  },

  optimization: {
    splitChunks: {
      chunks: 'all',
      cacheGroups: {
        vendor: {
          test: /[\\/]node_modules[\\/](react|react-dom|lodash|axios)[\\/]/,
          name: 'vendor',
          chunks: 'all',
          priority: 20,
        },
        internal: {
          test: /[\\/]node_modules[\\/](@xq9zk7823|company-internal-utils|private-logger)[\\/]/,
          name: 'internal',
          chunks: 'all',
          priority: 10,
        },
        commons: {
          name: 'commons',
          minChunks: 2,
          chunks: 'all',
          priority: 5,
          reuseExistingChunk: true,
        },
      },
    },
    runtimeChunk: 'single',
  },

  plugins: [
    new HtmlWebpackPlugin({
      template: './public/index.html',
      title: 'Webpack 5 React - depfused Test Lab',
      minify: {
        collapseWhitespace: true,
        removeComments: true,
      },
    }),
  ],
};
