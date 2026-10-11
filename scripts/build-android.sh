#!/usr/bin/env bash
set -euo pipefail

: "${ANDROID_NDK_HOME:?Set ANDROID_NDK_HOME to an installed Android NDK}"

TARGET="aarch64-linux-android"
API_LEVEL="${ANDROID_API_LEVEL:-26}"
HOST_TAG="${ANDROID_NDK_HOST_TAG:-linux-x86_64}"
TOOLCHAIN="$ANDROID_NDK_HOME/toolchains/llvm/prebuilt/$HOST_TAG/bin"

if [[ ! -x "$TOOLCHAIN/aarch64-linux-android${API_LEVEL}-clang" ]]; then
  echo "Android NDK clang not found: $TOOLCHAIN/aarch64-linux-android${API_LEVEL}-clang" >&2
  exit 1
fi

export CARGO_TARGET_AARCH64_LINUX_ANDROID_LINKER="$TOOLCHAIN/aarch64-linux-android${API_LEVEL}-clang"
export CC_aarch64_linux_android="$TOOLCHAIN/aarch64-linux-android${API_LEVEL}-clang"
export AR_aarch64_linux_android="$TOOLCHAIN/llvm-ar"

cargo build --lib --target "$TARGET" "$@"

mkdir -p "android/app/src/main/jniLibs/arm64-v8a"
cp "target/$TARGET/debug/libdevptp.so" \
  "android/app/src/main/jniLibs/arm64-v8a/"

echo "Built android/app/src/main/jniLibs/arm64-v8a/libdevptp.so"
