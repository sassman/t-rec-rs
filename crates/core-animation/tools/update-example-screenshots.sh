#!/usr/bin/env bash
# Updates the screenshots for all examples that support automatic screenshot capture.
#
# Usage: ./tools/update-example-screenshots.sh
#
# Screenshots are saved to examples/screenshots/
# Examples are auto-detected by looking for `#[cfg(feature = "screenshot")]` in the source.

set -e

SCRIPT_DIR="$(cd "$(dirname "${BASH_SOURCE[0]}")" && pwd)"
CRATE_DIR="$(dirname "$SCRIPT_DIR")"

cd "$CRATE_DIR/../.."

echo "Updating example screenshots for core-animation..."
echo ""

# Find examples that have screenshot support (contain the feature gate)
EXAMPLES=$(grep -l 'cfg(feature = "screenshot")' crates/core-animation/examples/*.rs 2>/dev/null | xargs -I{} basename {} .rs | sort)

if [ -z "$EXAMPLES" ]; then
    echo "No examples with screenshot support found."
    exit 0
fi

echo "Found examples with screenshot support:"
for example in $EXAMPLES; do
    echo "  - $example"
done
echo ""

# Ensure screenshots directory exists
mkdir -p crates/core-animation/examples/screenshots

for example in $EXAMPLES; do
    echo "Running $example..."
    cargo run -p core-animation --example "$example" --features screenshot
    echo ""
done

echo "Done! Screenshots saved to crates/core-animation/examples/screenshots/"
echo ""
ls -la crates/core-animation/examples/screenshots/
