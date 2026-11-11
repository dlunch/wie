#!/usr/bin/env bash

set -euo pipefail

repo="repos/$GITHUB_REPOSITORY"
tag=$RELEASE_TAG

if [[ "$CHANNEL" == stable ]]; then
  if ! release_id=$(gh release view "$tag" --repo "$GITHUB_REPOSITORY" --json databaseId --jq '.databaseId' 2>/dev/null); then
    release=$(gh api --method POST "$repo/releases" -f tag_name="$tag" -f name="$tag" -F draft=true -F prerelease=false)
    release_id=$(jq -r '.id' <<< "$release")
  fi

  # The updater picks its installer by "<os>-<arch>-<bundle type>", falling back to "<os>-<arch>".
  platforms='{}'
  while read -r target asset; do
    platforms=$(jq --arg target "$target" --arg url "https://github.com/$GITHUB_REPOSITORY/releases/download/$tag/$asset" \
      --rawfile signature "release-assets/$asset.sig" '.[$target] = {url: $url, signature: ($signature | rtrimstr("\n"))}' <<< "$platforms")
  done <<EOF
windows-x86_64-nsis wie-$tag-windows-x86_64-unsigned-setup.exe
windows-x86_64-msi wie-$tag-windows-x86_64-unsigned.msi
windows-aarch64-nsis wie-$tag-windows-aarch64-unsigned-setup.exe
windows-aarch64-msi wie-$tag-windows-aarch64-unsigned.msi
linux-x86_64-appimage wie-$tag-linux-x86_64.AppImage
linux-aarch64-appimage wie-$tag-linux-aarch64.AppImage
linux-x86_64-deb wie-$tag-linux-x86_64.deb
linux-aarch64-deb wie-$tag-linux-aarch64.deb
darwin-x86_64 wie-$tag-macos-universal-unsigned.app.tar.gz
darwin-aarch64 wie-$tag-macos-universal-unsigned.app.tar.gz
EOF
  jq -n --arg version "${tag#v}" --argjson platforms "$platforms" '{version: $version, platforms: $platforms}' > release-assets/latest.json

  gh release upload "$tag" release-assets/* --clobber --repo "$GITHUB_REPOSITORY"

  generate_notes_args=(--method POST "$repo/releases/generate-notes" -f tag_name="$tag")
  if [[ -n "$PREVIOUS_TAG" ]]; then
    generate_notes_args+=(-f previous_tag_name="$PREVIOUS_TAG")
  fi
  notes=$(gh api "${generate_notes_args[@]}" --jq '.body')
  body="## Changes

$notes"
  gh api --method PATCH "$repo/releases/$release_id" -f name="$tag" -f body="$body" -F draft=false -F prerelease=false >/dev/null
else
  if ! release_id=$(gh release view "$tag" --repo "$GITHUB_REPOSITORY" --json databaseId --jq '.databaseId' 2>/dev/null); then
    release=$(gh api --method POST "$repo/releases" -f tag_name="$tag" -f target_commitish="$TARGET_SHA" -f name=nightly -F draft=true -F prerelease=true)
    release_id=$(jq -r '.id' <<< "$release")
  fi

  gh release upload "$tag" release-assets/* --clobber --repo "$GITHUB_REPOSITORY"

  body="## Nightly

- Target SHA: \`$TARGET_SHA\`"
  gh api --method PATCH "$repo/releases/$release_id" -f name=nightly -f body="$body" -F draft=false -F prerelease=true >/dev/null

  while IFS= read -r previous_nightly_tag; do
    gh release delete "$previous_nightly_tag" --cleanup-tag --yes --repo "$GITHUB_REPOSITORY"
  done < <(
    gh release list --repo "$GITHUB_REPOSITORY" --limit 100 --json tagName,isPrerelease \
      --jq ".[] | select(.isPrerelease and (.tagName == \"nightly\" or (.tagName | startswith(\"nightly-\"))) and .tagName != \"$tag\") | .tagName"
  )
fi
