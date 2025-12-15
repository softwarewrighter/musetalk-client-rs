#!/bin/bash
#
# Build the MuseTalk Docker image
#
# Usage: ./build.sh [--no-cache]
#

set -e

SCRIPT_DIR="$(cd "$(dirname "${BASH_SOURCE[0]}")" && pwd)"
IMAGE_NAME="musetalk-server"
IMAGE_TAG="latest"

cd "$SCRIPT_DIR"

echo "Building MuseTalk Docker image..."
echo "  Image: ${IMAGE_NAME}:${IMAGE_TAG}"
echo ""

# Check for --no-cache flag
BUILD_ARGS=""
if [[ "$1" == "--no-cache" ]]; then
    BUILD_ARGS="--no-cache"
    echo "  Building without cache"
fi

# Build the image
docker build $BUILD_ARGS \
    -t "${IMAGE_NAME}:${IMAGE_TAG}" \
    -f Dockerfile \
    .

echo ""
echo "Build complete!"
echo ""
echo "Image: ${IMAGE_NAME}:${IMAGE_TAG}"
echo ""
echo "Next steps:"
echo "  1. Download models: ./download_models.sh"
echo "  2. Run the server: ./run.sh"
