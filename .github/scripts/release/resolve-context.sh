#!/usr/bin/env bash

set -euo pipefail

target_sha=$(git rev-parse 'HEAD^{commit}')

if [[ "$GITHUB_REF" == "refs/heads/main" ]]; then
  channel=nightly
  base_version=$(jq -r '.version' wie_app/tauri.conf.json)
  # MSI prerelease identifiers must be numeric and at most 65535; epoch days keep nightly versions monotonic.
  app_version="${base_version}-$(($(date -u +%s) / 86400))"
  asset_identity=nightly
  if [[ "$GITHUB_EVENT_NAME" == workflow_dispatch ]]; then
    should_build=true
  elif [[ $(git rev-parse --verify 'refs/tags/nightly^{commit}' 2>/dev/null || true) == "$target_sha" ]]; then
    should_build=false
  else
    should_build=true
  fi
elif [[ "$GITHUB_REF" =~ ^refs/tags/(v(0|[1-9][0-9]*)\.(0|[1-9][0-9]*)\.(0|[1-9][0-9]*))$ ]]; then
  channel=stable
  tag=${BASH_REMATCH[1]}
  app_version=${tag#v}
  asset_identity=$tag
  should_build=true
else
  echo "Unsupported release ref: $GITHUB_REF" >&2
  exit 1
fi

{
  echo "asset_identity=$asset_identity"
  echo "channel=$channel"
  echo "run_number=$GITHUB_RUN_NUMBER"
  echo "should_build=$should_build"
  echo "target_sha=$target_sha"
  echo "app_version=$app_version"
} >> "$GITHUB_OUTPUT"
