#!/usr/bin/env bash
# Generates recordings (GIF and MP4) for all examples that support automatic recording.
#
# Usage: ./tools/update-example-screenshots.sh
#
# Outputs are saved to examples/screenshots/
# Examples are auto-detected by looking for `#[cfg(feature = "record")]` in the source.
#
# Prerequisites:
# - ImageMagick (brew install imagemagick)
# - ffmpeg (brew install ffmpeg)

set -e

SCRIPT_DIR="$(cd "$(dirname "${BASH_SOURCE[0]}")" && pwd)"
CRATE_DIR="$(dirname "$SCRIPT_DIR")"

cd "$CRATE_DIR/../.."

echo "Generating example recordings for core-animation..."
echo ""

# Find examples that have recording support (contain the feature gate)
EXAMPLES=$(grep -l 'cfg(feature = "record")' crates/core-animation/examples/*.rs 2>/dev/null | xargs -I{} basename {} .rs | sort)

if [ -z "$EXAMPLES" ]; then
    echo "No examples with recording support found."
    exit 0
fi

echo "Found examples with recording support:"
for example in $EXAMPLES; do
    echo "  - $example"
done
echo ""

# Ensure screenshots directory exists
mkdir -p crates/core-animation/examples/screenshots

for example in $EXAMPLES; do
    echo "Recording $example..."
    cargo run -p core-animation --release --example "$example" --features record
    echo ""
done

echo "Done! Recordings saved to crates/core-animation/examples/screenshots/"
echo ""
ls -la crates/core-animation/examples/screenshots/
