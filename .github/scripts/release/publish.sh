#!/usr/bin/env bash

set -euo pipefail

repo="repos/$GITHUB_REPOSITORY"
tag=$RELEASE_TAG

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
  if release=$(gh api "$repo/releases/tags/$tag" 2>/dev/null); then
    release_id=$(jq -r '.id' <<< "$release")
  else
    initial_body=$(warning_body)
    release=$(gh api --method POST "$repo/releases" -f tag_name="$tag" -f target_commitish="$TARGET_SHA" -f name="$tag" -f body="$initial_body" -F draft=true -F prerelease=false)
    release_id=$(jq -r '.id' <<< "$release")
  fi

  gh release upload "$tag" release-assets/* --clobber --repo "$GITHUB_REPOSITORY"

  generate_notes_args=(--method POST "$repo/releases/generate-notes" -f tag_name="$tag" -f target_commitish="$TARGET_SHA")
  if [[ -n "$PREVIOUS_TAG" ]]; then
    generate_notes_args+=(-f previous_tag_name="$PREVIOUS_TAG")
  fi
  notes=$(gh api "${generate_notes_args[@]}" --jq '.body')
  body="$(warning_body)

## Changes

$notes"
  gh api --method PATCH "$repo/releases/$release_id" -f name="$tag" -f body="$body" -F draft=false -F prerelease=false >/dev/null
else
  if [[ $(gh api "$repo/git/ref/heads/main" --jq '.object.sha') != "$TARGET_SHA" ]]; then
    echo "Skipping stale nightly before any release mutation"
    exit 0
  fi

  if release=$(gh api "$repo/releases/tags/$tag" 2>/dev/null); then
    release_id=$(jq -r '.id' <<< "$release")
  else
    initial_body=$(warning_body)
    release=$(gh api --method POST "$repo/releases" -f tag_name="$tag" -f target_commitish="$TARGET_SHA" -f name=nightly -f body="$initial_body" -F draft=true -F prerelease=true)
    release_id=$(jq -r '.id' <<< "$release")
  fi

  gh release upload "$tag" release-assets/* --clobber --repo "$GITHUB_REPOSITORY"

  body="$(warning_body)

## Nightly

- Target SHA: \`$TARGET_SHA\`"
  gh api --method PATCH "$repo/releases/$release_id" -f name=nightly -f body="$body" -F draft=false -F prerelease=true >/dev/null

  while IFS= read -r previous_nightly_tag; do
    gh release delete "$previous_nightly_tag" --cleanup-tag --yes --repo "$GITHUB_REPOSITORY"
  done < <(
    gh release list --repo "$GITHUB_REPOSITORY" --limit 100 --json tagName,isPrerelease \
      --jq ".[] | select(.isPrerelease and (.tagName == \"nightly\" or (.tagName | startswith(\"nightly-\"))) and .tagName != \"$tag\") | .tagName"
  )
fi
