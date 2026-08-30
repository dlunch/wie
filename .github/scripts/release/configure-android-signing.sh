#!/usr/bin/env bash
set -euo pipefail
umask 077

keystore_path="$RUNNER_TEMP/android-upload-key.jks"

printf '%s' "$ANDROID_KEYSTORE_BASE64" | base64 --decode > "$keystore_path"
echo "ANDROID_KEYSTORE_PATH=$keystore_path" >> "$GITHUB_ENV"

cat >> wie-app/gen/android/app/build.gradle.kts <<'EOF'

android {
    signingConfigs {
        create("release") {
            storeFile = file(System.getenv("ANDROID_KEYSTORE_PATH"))
            storePassword = System.getenv("ANDROID_KEYSTORE_PASSWORD")
            keyAlias = System.getenv("ANDROID_KEY_ALIAS")
            keyPassword = System.getenv("ANDROID_KEY_PASSWORD")
        }
    }
    buildTypes.getByName("release") {
        signingConfig = signingConfigs.getByName("release")
    }
}
EOF
