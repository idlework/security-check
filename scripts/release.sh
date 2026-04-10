#!/bin/bash
set -e

VERSION="${1:?Usage: ./scripts/release.sh 0.2.0}"
VERSION="${VERSION#v}" # strip leading v if present

# Update Cargo.toml
sed -i '' "s/^version = \".*\"/version = \"${VERSION}\"/" Cargo.toml

# Verify it builds
cargo build --release

# Commit, tag, push
git add Cargo.toml
git commit -m "Release v${VERSION}"
git tag "v${VERSION}"
git push && git push origin "v${VERSION}"

echo "Released v${VERSION} — CI will create the GitHub release."
