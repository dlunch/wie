#!/usr/bin/env bash

set -euo pipefail

repo="repos/$GITHUB_REPOSITORY"

warning_body() {
  cat <<EOF
## Unsigned artifacts

- The Windows, Linux, macOS, Android, and iOS artifacts are unsigned.
- The macOS artifacts are not notarized.
- Android AAB files cannot be installed directly on a device.
- The iOS \`.xcarchive.zip\` is not an IPA for device installation, TestFlight, or App Store submission.
EOF
}

if [[ "$CHANNEL" == stable ]]; then
  tag=${GITHUB_REF#refs/tags/}
  if release=$(gh api "$repo/releases/tags/$tag" 2>/dev/null); then
    release_id=$(jq -r '.id' <<< "$release")
  else
    initial_body=$(warning_body)
    release=$(gh api --method POST "$repo/releases" -f tag_name="$tag" -f target_commitish="$TARGET_SHA" -f name="$tag" -f body="$initial_body" -F draft=true -F prerelease=false)
    release_id=$(jq -r '.id' <<< "$release")
  fi

  gh release upload "$tag" release-assets/* --clobber --repo "$GITHUB_REPOSITORY"

  notes=$(gh api --method POST "$repo/releases/generate-notes" -f tag_name="$tag" -f target_commitish="$TARGET_SHA" --jq '.body')
  body="$(warning_body)

## Changes

$notes"
  gh api --method PATCH "$repo/releases/$release_id" -f name="$tag" -f body="$body" -F draft=false -F prerelease=false >/dev/null
else
  if [[ $(gh api "$repo/git/ref/heads/main" --jq '.object.sha') != "$TARGET_SHA" ]]; then
    echo "Skipping stale nightly before any release mutation"
    exit 0
  fi

  if release=$(gh api "$repo/releases/tags/nightly" 2>/dev/null); then
    release_id=$(jq -r '.id' <<< "$release")
  else
    initial_body=$(warning_body)
    release=$(gh api --method POST "$repo/releases" -f tag_name=nightly -f target_commitish="$TARGET_SHA" -f name=nightly -f body="$initial_body" -F draft=true -F prerelease=true)
    release_id=$(jq -r '.id' <<< "$release")
  fi

  gh release upload nightly release-assets/* --clobber --repo "$GITHUB_REPOSITORY"

  if gh api "$repo/git/ref/tags/nightly" >/dev/null 2>&1; then
    gh api --method PATCH "$repo/git/refs/tags/nightly" -f sha="$TARGET_SHA" -F force=true >/dev/null
  else
    gh api --method POST "$repo/git/refs" -f ref=refs/tags/nightly -f sha="$TARGET_SHA" >/dev/null
  fi
  body="$(warning_body)

## Nightly

- Target SHA: \`$TARGET_SHA\`
- Workflow run: \`$RUN_NUMBER\`"
  gh api --method PATCH "$repo/releases/$release_id" -f name=nightly -f body="$body" -F draft=false -F prerelease=true >/dev/null
fi
