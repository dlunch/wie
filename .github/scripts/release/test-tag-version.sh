#!/usr/bin/env bash

set -euo pipefail

script="$(cd "$(dirname "${BASH_SOURCE[0]}")" && pwd)/tag-version.sh"
temp_dir=$(mktemp -d)
trap 'rm -rf "$temp_dir"' EXIT
trap 'echo "FAIL: line $LINENO" >&2' ERR
export GIT_CONFIG_GLOBAL=/dev/null
export CARGO_HOME="$temp_dir/cargo"
export CARGO_TARGET_DIR="$temp_dir/target"
export CARGO_NET_OFFLINE=true
export GITHUB_REPOSITORY=example/wie GH_TOKEN=fixture-only
export PATH="$temp_dir/bin:$PATH"

mkdir -p "$temp_dir/bin"
cat > "$temp_dir/bin/gh" <<'EOF'
#!/usr/bin/env bash
set -euo pipefail
printf '%s\n' "$@" >> "$GH_LOG"
EOF
chmod +x "$temp_dir/bin/gh"

for versions in '1.2.3 1.2.4 2' '0.1.5 0.1.6 1'; do
  read -r release_version next_version commit_count <<< "$versions"
  remote="$temp_dir/remote-$release_version.git"
  git init --quiet --bare --initial-branch=main "$remote"
  git init --quiet --initial-branch=main "$temp_dir/work-$release_version"
  cd "$temp_dir/work-$release_version"
  git config user.name 'Release Test'
  git config user.email 'release-test@example.invalid'
  git remote add origin "$remote"
  export GH_LOG="$temp_dir/gh-$release_version.log"
  mkdir -p src wie-app/src
  cat > Cargo.toml <<'EOF'
[workspace]
members = ["wie-app"]
resolver = "2"

[workspace.package]
version = "0.1.5"

[package]
name = "wie"
version.workspace = true
edition = "2024"
EOF
  cat > wie-app/Cargo.toml <<'EOF'
[package]
name = "wie-app"
version.workspace = true
edition = "2024"

[dependencies]
wie = { path = "..", version = "0.1.5" }
EOF
  printf 'pub const VERSION: &str = env!("CARGO_PKG_VERSION");\n' > src/lib.rs
  printf 'pub use wie::VERSION;\n' > wie-app/src/lib.rs
  cargo fmt --all
  cargo clippy --workspace -- -D warnings
  git add Cargo.toml Cargo.lock src wie-app
  git commit --quiet -m 'Seed workspace'
  seed=$(git rev-parse HEAD)
  git tag nightly-existing
  git push --quiet origin main refs/tags/nightly-existing
  git tag nightly-local
  printf 'Must not be committed\n' > unrelated.txt

  RELEASE_VERSION=$release_version bash "$script"

  [[ $(git symbolic-ref --short HEAD) == "release/v$release_version" ]]
  [[ $(git rev-list --count main..HEAD) == "$commit_count" ]]
  [[ $(git rev-parse HEAD^) == "$(git rev-parse "v$release_version")" ]]
  [[ $(git --git-dir="$remote" rev-parse "refs/heads/release/v$release_version") == "$(git rev-parse HEAD)" ]]
  diff -u <(printf '%s\n' Cargo.lock Cargo.toml wie-app/Cargo.toml) <(git diff --name-only main HEAD)
  diff -u <(printf '%s\n' \
    "$seed refs/heads/main" \
    "$(git rev-parse HEAD) refs/heads/release/v$release_version" \
    "$seed refs/tags/nightly-existing" \
    "$(git rev-parse "v$release_version") refs/tags/v$release_version" | sort) \
    <(git --git-dir="$remote" show-ref | sort)

  for revision in "v$release_version $release_version" "release/v$release_version $next_version"; do
    read -r ref expected_version <<< "$revision"
    git switch --quiet --detach "$ref"
    cargo metadata --locked --offline --format-version 1 | jq -e --arg version "$expected_version" \
      '(.packages | length == 2) and all(.packages[]; .version == $version)' > /dev/null
  done
  GITHUB_REF=refs/heads/main GITHUB_EVENT_NAME=workflow_dispatch GITHUB_RUN_NUMBER=1 \
    GITHUB_OUTPUT="$temp_dir/context-$release_version" bash "${script%/*}/resolve-context.sh"
  grep -Fq "app_version=$next_version-" "$temp_dir/context-$release_version"
  diff -u <(printf '%s\n' \
    workflow run release.yaml --ref "v$release_version" --repo "$GITHUB_REPOSITORY" \
    pr create --base main --head "release/v$release_version" --repo "$GITHUB_REPOSITORY" \
    --title "Start $next_version development" \
    --body "Update Cargo workspace manifests and Cargo.lock to $next_version for development.

Verified with cargo fmt --all and cargo clippy --workspace -- -D warnings before each commit.") "$GH_LOG"
  echo "PASS: release $release_version, development $next_version, scoped push, GitHub calls and nightly resolver"

  remote_refs=$(git --git-dir="$remote" show-ref)
  gh_calls=$(< "$GH_LOG")
  git switch --quiet main
  for invalid_version in '01.2.3' '1.2.3-rc.1'; do
    if RELEASE_VERSION=$invalid_version bash "$script" > "$temp_dir/invalid.log" 2>&1; then
      echo "Accepted invalid RELEASE_VERSION: $invalid_version" >&2
      exit 1
    fi
    grep -q RELEASE_VERSION "$temp_dir/invalid.log"
    [[ $(git --git-dir="$remote" show-ref) == "$remote_refs" ]]
    [[ $(< "$GH_LOG") == "$gh_calls" ]]
  done
  echo 'PASS: invalid stable versions leave remote refs and GitHub calls unchanged'

  git branch --quiet -D "release/v$release_version"
  if RELEASE_VERSION=$release_version bash "$script" > "$temp_dir/duplicate.log" 2>&1; then
    echo "Overwrote existing tag: v$release_version" >&2
    exit 1
  fi
  grep -q "tag 'v$release_version' already exists" "$temp_dir/duplicate.log"
  [[ $(git --git-dir="$remote" show-ref) == "$remote_refs" ]]
  [[ $(< "$GH_LOG") == "$gh_calls" ]]
  echo 'PASS: duplicate tag fails before remote or GitHub mutations'
done
