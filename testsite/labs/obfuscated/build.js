const esbuild = require('esbuild');

async function build() {
  // Build the main bundle
  await esbuild.build({
    entryPoints: ['src/index.js'],
    bundle: true,
    outfile: 'dist/bundle.js',
    format: 'cjs',
    platform: 'node',
    target: 'node18',
    sourcemap: false,
    minify: false
  });
  console.log('Built dist/bundle.js');

  // Build the manual obfuscated file
  await esbuild.build({
    entryPoints: ['src/manual-obfuscated.js'],
    bundle: true,
    outfile: 'dist/manual-obfuscated.js',
    format: 'cjs',
    platform: 'node',
    target: 'node18',
    sourcemap: false,
    minify: false
  });
  console.log('Built dist/manual-obfuscated.js');
}

build().catch(err => {
  console.error(err);
  process.exit(1);
});
