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

cargo set-version --workspace --bump patch
next_version=$(cargo metadata --no-deps --format-version 1 | jq -r '.packages[] | select(.name == "wie-app") | .version')
git add -- Cargo.toml Cargo.lock ':(glob)**/Cargo.toml'
git commit -m "Start $next_version development"

git push --atomic origin HEAD:refs/heads/main "refs/tags/$tag"
