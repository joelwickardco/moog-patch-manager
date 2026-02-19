#!/bin/bash
set -e

VERSION=$1

if [ -z "$VERSION" ]; then
  echo "Usage: ./version-bump.sh <version>"
  echo "Example: ./version-bump.sh 1.0.6"
  exit 1
fi

echo "Bumping version to $VERSION..."

npm version "$VERSION" --no-git-tag-version
sed -i '' "s/^version = \".*\"/version = \"$VERSION\"/" src-tauri/Cargo.toml
cargo update --workspace --manifest-path src-tauri/Cargo.toml
echo "$VERSION" > version.txt

echo "Done. All files updated to $VERSION."
