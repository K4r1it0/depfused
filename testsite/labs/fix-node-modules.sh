#!/bin/bash
# Fix all labs: copy fake packages into node_modules/@xq9zk7823/ so bundlers
# resolve them as real npm packages (matching real-world behavior).
# This produces source maps with node_modules/@xq9zk7823/pkg paths.

LABS_DIR="$(cd "$(dirname "$0")" && pwd)"
PACKAGES_DIR="$LABS_DIR/packages"

fix_lab() {
    local lab_dir="$1"
    local lab_name="$(basename "$lab_dir")"

    if [ ! -d "$lab_dir/node_modules" ]; then
        echo "SKIP $lab_name: no node_modules"
        return
    fi

    echo "Fixing $lab_name..."

    # Create @xq9zk7823 scope in node_modules
    mkdir -p "$lab_dir/node_modules/@xq9zk7823"

    # Copy scoped packages
    for pkg_dir in "$PACKAGES_DIR"/xq9zk7823-*; do
        local pkg_name=$(basename "$pkg_dir" | sed 's/^xq9zk7823-//')
        local target="$lab_dir/node_modules/@xq9zk7823/$pkg_name"

        # Remove symlink if exists, then copy
        rm -rf "$target"
        cp -r "$pkg_dir" "$target"

        # Fix the package.json name to match scoped format
        if [ -f "$target/package.json" ]; then
            # Already has correct name from our setup
            :
        fi
    done

    # Copy unscoped packages
    for pkg in company-internal-utils private-logger enterprise-sdk; do
        if [ -d "$PACKAGES_DIR/$pkg" ]; then
            local target="$lab_dir/node_modules/$pkg"
            rm -rf "$target"
            cp -r "$PACKAGES_DIR/$pkg" "$target"
        fi
    done

    echo "  Done: installed fake packages into node_modules/"
}

# Fix each lab
for lab in webpack5-react vite-vue parcel-react esbuild-app rollup-library swc-app angular-app obfuscated nextjs-app; do
    if [ -d "$LABS_DIR/$lab" ]; then
        fix_lab "$LABS_DIR/$lab"
    fi
done

echo ""
echo "All labs fixed. Now rebuild each lab to get correct source maps."
