# depfused

High-performance dependency confusion scanner. Detects vulnerable private packages in production websites by capturing JavaScript bundles via headless browser, extracting package names through AST parsing and source map analysis, and checking them against the npm registry.

## Features

- **Headless browser capture** -- launches Chromium to intercept all JS files, including dynamically loaded chunks
- **Multi-parser extraction** -- AST parsing (oxc), source maps, webpack/vite/parcel/esbuild/rollup/SWC/Angular patterns, deobfuscation
- **9-layer false positive filtering** -- eliminates CSS classes, URLs, i18n keys, bundler internals, and other non-package artifacts
- **npm registry checks** -- verifies package existence, detects unclaimed scopes (Critical), and missing unscoped packages (High/Medium)
- **Self-contained browser** -- automatically downloads Chromium if not installed (`depfused setup`)
- **Parallel host-grouped scanning** -- reuses browser instances per host, scans multiple targets concurrently
- **Telegram notifications** -- optional alerts for high-severity findings

## Installation

### From source

```bash
cargo install --path .
```

### Pre-built binary

Download from the [Releases](../../releases) page.

### Browser setup

depfused needs Chromium to capture JavaScript. It will auto-download one on first scan, or you can pre-install:

```bash
depfused setup
```

If you already have Chrome/Chromium installed, depfused will find it automatically. You can also specify a path explicitly:

```bash
depfused scan https://example.com --chrome-path /usr/bin/chromium
```

## Usage

### Scan a single target

```bash
depfused scan https://example.com
```

### Scan multiple targets from file

```bash
depfused scan -f urls.txt -p 4
```

### JSON output

```bash
depfused scan https://example.com --json -o results.json
```

### All options

```
depfused scan [OPTIONS] [TARGETS]...

Arguments:
  [TARGETS]...  Target URL(s) to scan

Options:
  -f, --file <FILE>              File containing URLs (one per line)
  -p, --parallel <N>             Number of sites to scan in parallel [default: 1]
  -o, --output <FILE>            Output file path
      --json                     Output as JSON
      --fast                     Fast mode: reduce wait times (may miss lazy-loaded JS)
  -q, --quiet                    Only show targets with vulnerabilities
      --scoped-only              Only check scoped packages (@scope/pkg)
      --skip-npm-check           Only extract packages, skip npm verification
      --min-confidence <LEVEL>   Minimum confidence: low, medium, high [default: low]
      --chrome-path <PATH>       Path to Chrome/Chromium executable
      --timeout <SECS>           Request timeout [default: 30]
      --rate-limit <RPS>         Rate limit (requests/sec) [default: 10]
      --telegram                 Enable Telegram notifications
  -v, --verbose                  Verbose output
  -h, --help                     Print help
```

## How it works

1. **Browser capture** -- navigates to the target URL in headless Chromium, intercepts all JavaScript responses via CDP
2. **Lazy chunk discovery** -- finds chunk URLs referenced in captured JS and fetches them (up to 3 depth levels)
3. **Source map probing** -- attempts to fetch `.map` files even when not explicitly referenced
4. **Package extraction** -- runs 5 extraction methods in parallel per JS file:
   - AST parsing (import/require/dynamic import statements)
   - Source map `sources` array parsing
   - Webpack chunk manifest extraction
   - Bundler-specific pattern matching (Vite, Parcel, esbuild, Rollup, SWC, Turbopack)
   - Deobfuscation (base64, hex, unicode, char codes, array joins)
5. **False positive filtering** -- 9 filter layers remove artifacts that look like packages but aren't
6. **npm registry verification** -- checks each extracted package:
   - **Exists** -- package is public on npm (Info)
   - **NotFound** (unscoped) -- package doesn't exist, could be registered by attacker (Medium/High)
   - **ScopeNotClaimed** -- the `@scope` itself doesn't exist on npm, attacker can claim it (Critical)

## Severity levels

| Severity | Meaning |
|----------|---------|
| **Critical** | Scoped package with unclaimed scope -- attacker can register the scope and publish |
| **High** | Unscoped package not on npm, name suggests internal use |
| **Medium** | Unscoped package not on npm |
| **Info** | Package exists on npm (not vulnerable) |

## Performance

Benchmarked scanning 9 test targets on localhost:

| Mode | Time | Speedup |
|------|------|---------|
| `-p 1` (sequential) | 29.3s | 1x |
| `-p 4` (4 parallel) | 8.1s | 3.6x |
| `-p 9` (9 parallel) | 5.7s | 5.1x |

Host grouping reuses browser instances for URLs on the same host, reducing overhead.

## Test labs

The `testsite/labs/` directory contains 9 test applications covering all major bundlers:

| Lab | Bundler | Framework |
|-----|---------|-----------|
| webpack5-react | Webpack 5 | React |
| vite-vue | Vite | Vue 3 |
| parcel-react | Parcel | React |
| esbuild-app | esbuild | Vanilla |
| rollup-library | Rollup | Library |
| swc-app | SWC | React |
| angular-app | Angular | Angular |
| obfuscated | Webpack + obfuscation | React |
| nextjs-app | Next.js (Webpack) | React |

To run the labs:

```bash
cd testsite/labs
./fix-node-modules.sh   # Install fake packages into node_modules
python3 serve-all.py     # Serves labs on ports 9001-9009
```

Then scan them:

```bash
depfused scan -f urls.txt -p 4 --json -o results.json
```

## Building

```bash
# Debug
cargo build

# Release
cargo build --release

# Linux x86_64 (requires cross)
cross build --release --target x86_64-unknown-linux-gnu

# Run tests
cargo test
```

## License

MIT
