#!/bin/bash
# Build script for Contexture Music Player Plugin (Gradle + Kotlin)

set -e

echo "Building Contexture Music Player Plugin with Gradle..."

# Check if Rust core library exists
RUST_LIB_DIR="../target/release"
if [ ! -d "$RUST_LIB_DIR" ]; then
    echo "Error: Rust core library not found. Please build the Rust core first:"
    echo "  cd core && cargo build --release"
    exit 1
fi

# Detect platform and copy native library
OS_NAME=$(uname -s)
OS_ARCH=$(uname -m)

case "$OS_NAME" in
    Linux*)
        PLATFORM="linux-x64"
        LIB_NAME="libcontextune_core.so"
        ;;
    Darwin*)
        if [ "$OS_ARCH" = "arm64" ]; then
            PLATFORM="macos-aarch64"
        else
            PLATFORM="macos-x64"
        fi
        LIB_NAME="libcontextune_core.dylib"
        ;;
    MINGW*|MSYS*|CYGWIN*)
        PLATFORM="windows-x64"
        LIB_NAME="contextune_core.dll"
        ;;
    *)
        echo "Error: Unsupported platform: $OS_NAME"
        exit 1
        ;;
esac

echo "Platform detected: $PLATFORM"

# Create libs directory
mkdir -p "libs/$PLATFORM"

# Copy native library
if [ -f "$RUST_LIB_DIR/$LIB_NAME" ]; then
    echo "Copying native library: $LIB_NAME"
    cp "$RUST_LIB_DIR/$LIB_NAME" "libs/$PLATFORM/"
else
    echo "Error: Native library not found: $RUST_LIB_DIR/$LIB_NAME"
    exit 1
fi

# Build plugin using Gradle wrapper
echo "Building plugin with Gradle..."
./gradlew buildPlugin

echo ""
echo "Build complete! Plugin ZIP file is in: build/distributions/"
echo ""
echo "To run the plugin in development mode:"
echo "  ./gradlew runIde"
