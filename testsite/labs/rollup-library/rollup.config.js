import resolve from '@rollup/plugin-node-resolve';
import commonjs from '@rollup/plugin-commonjs';

export default {
  input: 'src/index.js',
  output: [
    {
      file: 'dist/index.esm.js',
      format: 'es',
      sourcemap: true,
      exports: 'named'
    },
    {
      file: 'dist/index.cjs.js',
      format: 'cjs',
      sourcemap: true,
      exports: 'named'
    },
    {
      file: 'dist/browser/bundle.js',
      format: 'iife',
      name: 'RollupLib',
      sourcemap: true
    }
  ],
  plugins: [
    resolve({
      preferBuiltins: false
    }),
    commonjs()
  ]
};
