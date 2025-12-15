#!/bin/bash
#
# Stop the MuseTalk inference server
#

CONTAINER_NAME="musetalk"

if docker ps -q -f name="$CONTAINER_NAME" | grep -q .; then
    echo "Stopping MuseTalk server..."
    docker stop "$CONTAINER_NAME"
    echo "Server stopped."
else
    echo "MuseTalk server is not running."
fi
