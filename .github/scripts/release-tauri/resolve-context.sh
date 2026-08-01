#!/usr/bin/env bash

set -euo pipefail

target_sha=$(git rev-parse 'HEAD^{commit}')

if [[ "$GITHUB_REF" == "refs/heads/main" ]]; then
  channel=nightly
  base_version=$(jq -r '.version' wie_app/tauri.conf.json)
  app_version="${base_version}-nightly-${target_sha:0:12}"
  asset_identity=nightly
elif [[ "$GITHUB_REF" =~ ^refs/tags/(v(0|[1-9][0-9]*)\.(0|[1-9][0-9]*)\.(0|[1-9][0-9]*))$ ]]; then
  channel=stable
  tag=${BASH_REMATCH[1]}
  app_version=${tag#v}
  asset_identity=$tag
else
  echo "Unsupported release ref: $GITHUB_REF" >&2
  exit 1
fi

{
  echo "asset_identity=$asset_identity"
  echo "channel=$channel"
  echo "run_number=$GITHUB_RUN_NUMBER"
  echo "target_sha=$target_sha"
  echo "app_version=$app_version"
} >> "$GITHUB_OUTPUT"
