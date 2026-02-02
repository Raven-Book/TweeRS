#!/bin/bash
# Build script for compiling tweers-core to WASM

set -e

# Colors for output
RED='\033[0;31m'
GREEN='\033[0;32m'
YELLOW='\033[1;33m'
NC='\033[0m' # No Color

echo -e "${GREEN}=== TweeRS WASM Build Script ===${NC}"
echo ""

# Check if wasm-pack is installed
if ! command -v wasm-pack &> /dev/null; then
    echo -e "${RED}Error: wasm-pack is not installed${NC}"
    echo "Install it with: curl https://rustwasm.github.io/wasm-pack/installer/init.sh -sSf | sh"
    exit 1
fi

# Check if wasm32 target is installed
if ! rustup target list | grep -q "wasm32-unknown-unknown (installed)"; then
    echo -e "${YELLOW}Installing wasm32-unknown-unknown target...${NC}"
    rustup target add wasm32-unknown-unknown
fi

# Parse arguments
TARGET="web"
PROFILE="release"
OUT_DIR="target/wasm"

while [[ $# -gt 0 ]]; do
    case $1 in
        --target)
            TARGET="$2"
            shift 2
            ;;
        --dev)
            PROFILE="dev"
            shift
            ;;
        --out-dir)
            OUT_DIR="$2"
            shift 2
            ;;
        --help)
            echo "Usage: $0 [OPTIONS]"
            echo ""
            echo "Options:"
            echo "  --target <TARGET>    Target platform: web, nodejs, bundler (default: web)"
            echo "  --dev                Build in dev mode (default: release)"
            echo "  --out-dir <DIR>      Output directory (default: pkg)"
            echo "  --help               Show this help message"
            exit 0
            ;;
        *)
            echo -e "${RED}Unknown option: $1${NC}"
            exit 1
            ;;
    esac
done

echo -e "${GREEN}Building with:${NC}"
echo "  Target: $TARGET"
echo "  Profile: $PROFILE"
echo "  Output: $OUT_DIR"
echo ""

# Navigate to core crate
cd crates/core

# Build command - wasm-pack outputs to pkg/ by default
BUILD_CMD="wasm-pack build --target $TARGET --features wasm"

if [ "$PROFILE" = "dev" ]; then
    BUILD_CMD="$BUILD_CMD --dev"
fi
# Note: wasm-pack builds in release mode by default

echo -e "${YELLOW}Running: $BUILD_CMD${NC}"
echo ""

# Execute build
eval $BUILD_CMD

# Move output to target directory
echo ""
echo -e "${YELLOW}Moving output to ../../$OUT_DIR${NC}"
mkdir -p "../../$OUT_DIR"
rm -rf "../../$OUT_DIR"/*
cp -r pkg/* "../../$OUT_DIR/"
rm -rf pkg

# Replace auto-generated TypeScript definitions with custom ones
echo -e "${YELLOW}Replacing TypeScript definitions with custom types${NC}"
if [ -f "src/wasm/tweers_core.d.ts" ]; then
    cp src/wasm/tweers_core.d.ts "../../$OUT_DIR/tweers_core.d.ts"
    echo -e "${GREEN}✓ Custom TypeScript definitions applied${NC}"
else
    echo -e "${YELLOW}⚠ Custom TypeScript definitions not found, using auto-generated${NC}"
fi

echo ""
echo -e "${GREEN}✓ Build completed successfully!${NC}"
echo -e "Output directory: ${YELLOW}$OUT_DIR${NC}"
echo ""
echo "Files generated:"
ls -lh "../../$OUT_DIR" | grep -E '\.(js|wasm|ts)$' || true
