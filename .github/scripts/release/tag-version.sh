#!/usr/bin/env bash

set -euo pipefail

: "${RELEASE_VERSION:?RELEASE_VERSION is required}"
if [[ ! "$RELEASE_VERSION" =~ ^(0|[1-9][0-9]*)\.(0|[1-9][0-9]*)\.(0|[1-9][0-9]*)$ ]]; then
  echo "Invalid RELEASE_VERSION: $RELEASE_VERSION (expected stable X.Y.Z)" >&2
  exit 1
fi

tag="v$RELEASE_VERSION"

cargo set-version --workspace "$RELEASE_VERSION"
git add -- Cargo.toml Cargo.lock ':(glob)**/Cargo.toml'
if ! git diff --cached --quiet; then
  git commit -m "Release $RELEASE_VERSION"
fi
git tag "$tag"

git push --atomic origin HEAD:refs/heads/main "refs/tags/$tag"
