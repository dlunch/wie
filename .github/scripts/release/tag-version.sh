#!/usr/bin/env bash

set -euo pipefail

: "${RELEASE_VERSION:?RELEASE_VERSION is required}"
if [[ ! "$RELEASE_VERSION" =~ ^(0|[1-9][0-9]*)\.(0|[1-9][0-9]*)\.(0|[1-9][0-9]*)$ ]]; then
  echo "Invalid RELEASE_VERSION: $RELEASE_VERSION (expected stable X.Y.Z)" >&2
  exit 1
fi

tag="v$RELEASE_VERSION"
branch="release/$tag"
git switch -c "$branch"

cargo set-version --workspace "$RELEASE_VERSION"
cargo fmt --all
cargo clippy --workspace -- -D warnings
git add -- Cargo.toml Cargo.lock ':(glob)**/Cargo.toml'
if ! git diff --cached --quiet; then
  git commit -m "Release $RELEASE_VERSION"
fi
git tag "$tag"

cargo set-version --workspace --bump patch
cargo fmt --all
cargo clippy --workspace -- -D warnings
next_version=$(cargo metadata --no-deps --format-version 1 | jq -r '.packages[] | select(.name == "wie-app") | .version')
git add -- Cargo.toml Cargo.lock ':(glob)**/Cargo.toml'
git commit -m "Start $next_version development"

git push --atomic origin "HEAD:refs/heads/$branch" "refs/tags/$tag"
gh workflow run release.yaml --ref "$tag" --repo "$GITHUB_REPOSITORY"
gh pr create --base main --head "$branch" --repo "$GITHUB_REPOSITORY" \
  --title "Start $next_version development" \
  --body "Update Cargo workspace manifests and Cargo.lock to $next_version for development.

Verified with cargo fmt --all and cargo clippy --workspace -- -D warnings before each commit."
