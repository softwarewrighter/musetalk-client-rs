#!/bin/bash
#
# Run the MuseTalk inference server
#
# Usage: ./run.sh [--detach] [--port PORT]
#
# Environment variables:
#   MUSETALK_MODELS_DIR  - Path to models directory (default: ./models)
#   MUSETALK_PORT        - Server port (default: 8000)
#

set -e

SCRIPT_DIR="$(cd "$(dirname "${BASH_SOURCE[0]}")" && pwd)"
IMAGE_NAME="musetalk-server"
IMAGE_TAG="latest"
CONTAINER_NAME="musetalk"

# Default values
MODELS_DIR="${MUSETALK_MODELS_DIR:-$SCRIPT_DIR/models}"
PORT="${MUSETALK_PORT:-8000}"
DETACH=""

# Parse arguments
while [[ $# -gt 0 ]]; do
    case $1 in
        --detach|-d)
            DETACH="-d"
            shift
            ;;
        --port|-p)
            PORT="$2"
            shift 2
            ;;
        --models)
            MODELS_DIR="$2"
            shift 2
            ;;
        --help|-h)
            echo "Usage: ./run.sh [OPTIONS]"
            echo ""
            echo "Options:"
            echo "  -d, --detach     Run in background"
            echo "  -p, --port PORT  Server port (default: 8000)"
            echo "  --models DIR     Path to models directory"
            echo "  -h, --help       Show this help"
            echo ""
            echo "Environment variables:"
            echo "  MUSETALK_MODELS_DIR  Path to models directory"
            echo "  MUSETALK_PORT        Server port"
            exit 0
            ;;
        *)
            echo "Unknown option: $1"
            exit 1
            ;;
    esac
done

# Check if models directory exists
if [[ ! -d "$MODELS_DIR" ]]; then
    echo "Error: Models directory not found: $MODELS_DIR"
    echo ""
    echo "Please download models first:"
    echo "  ./download_models.sh"
    exit 1
fi

# Check if image exists
if ! docker image inspect "${IMAGE_NAME}:${IMAGE_TAG}" &>/dev/null; then
    echo "Error: Docker image not found: ${IMAGE_NAME}:${IMAGE_TAG}"
    echo ""
    echo "Please build the image first:"
    echo "  ./build.sh"
    exit 1
fi

# Stop existing container if running
if docker ps -q -f name="$CONTAINER_NAME" | grep -q .; then
    echo "Stopping existing container..."
    docker stop "$CONTAINER_NAME" >/dev/null
fi

# Remove existing container
if docker ps -aq -f name="$CONTAINER_NAME" | grep -q .; then
    docker rm "$CONTAINER_NAME" >/dev/null
fi

echo "Starting MuseTalk server..."
echo "  Port: $PORT"
echo "  Models: $MODELS_DIR"
echo ""

# Run the container
docker run \
    $DETACH \
    --name "$CONTAINER_NAME" \
    --gpus all \
    --shm-size=8g \
    -p "${PORT}:8000" \
    -v "${MODELS_DIR}:/app/musetalk/models:ro" \
    -e "PORT=8000" \
    -e "PRELOAD_MODELS=false" \
    --restart unless-stopped \
    "${IMAGE_NAME}:${IMAGE_TAG}"

if [[ -n "$DETACH" ]]; then
    echo ""
    echo "Server started in background."
    echo ""
    echo "Check status:  docker logs -f $CONTAINER_NAME"
    echo "Stop server:   docker stop $CONTAINER_NAME"
    echo ""
    echo "Server URL: http://localhost:$PORT"
fi
