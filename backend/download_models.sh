#!/bin/bash
#
# Download MuseTalk model weights
#
# Models are downloaded from HuggingFace to ./models directory
#
# Required models:
#   - musetalk or musetalkV15 (main model)
#   - dwpose (pose detection)
#   - face-parse-bisent (face parsing)
#   - sd-vae (VAE decoder)
#   - whisper (audio encoding)
#   - syncnet (optional, for sync scoring)
#

set -e

SCRIPT_DIR="$(cd "$(dirname "${BASH_SOURCE[0]}")" && pwd)"
MODELS_DIR="${MUSETALK_MODELS_DIR:-$SCRIPT_DIR/models}"

echo "MuseTalk Model Downloader"
echo "========================="
echo ""
echo "Models will be downloaded to: $MODELS_DIR"
echo ""

# Create models directory
mkdir -p "$MODELS_DIR"

# Check for required tools
if ! command -v wget &>/dev/null && ! command -v curl &>/dev/null; then
    echo "Error: wget or curl required"
    exit 1
fi

# Function to download with wget or curl
download() {
    local url="$1"
    local output="$2"

    if command -v wget &>/dev/null; then
        wget -q --show-progress -O "$output" "$url"
    else
        curl -L --progress-bar -o "$output" "$url"
    fi
}

# Check if git-lfs is available (preferred for HuggingFace)
if command -v git-lfs &>/dev/null; then
    HAS_GIT_LFS=true
else
    HAS_GIT_LFS=false
    echo "Note: git-lfs not found. Using direct downloads (slower)."
    echo ""
fi

echo "Downloading models from HuggingFace..."
echo ""

# Clone the models repo using git-lfs if available
if $HAS_GIT_LFS; then
    echo "[1/6] Cloning MuseTalk models (this may take a while)..."

    if [[ -d "$MODELS_DIR/.git" ]]; then
        echo "  Models repo already cloned, pulling updates..."
        cd "$MODELS_DIR"
        git pull
        cd "$SCRIPT_DIR"
    else
        # Clone TMElyralab/MuseTalk-Models or equivalent
        git clone https://huggingface.co/TMElyralab/MuseTalk "$MODELS_DIR/musetalk-repo" || true
    fi
else
    echo "Manual download required. Please download models from:"
    echo ""
    echo "  https://huggingface.co/TMElyralab/MuseTalk"
    echo ""
    echo "And extract to: $MODELS_DIR"
    echo ""
    echo "Required directory structure:"
    echo "  models/"
    echo "    musetalk/ or musetalkV15/"
    echo "    dwpose/"
    echo "    face-parse-bisent/"
    echo "    sd-vae/"
    echo "    whisper/"
    echo ""
fi

# Create expected directory structure
mkdir -p "$MODELS_DIR/musetalk"
mkdir -p "$MODELS_DIR/dwpose"
mkdir -p "$MODELS_DIR/face-parse-bisent"
mkdir -p "$MODELS_DIR/sd-vae"
mkdir -p "$MODELS_DIR/whisper"
mkdir -p "$MODELS_DIR/syncnet"

echo ""
echo "Model download instructions:"
echo "============================"
echo ""
echo "Due to HuggingFace's file sizes, you may need to manually download:"
echo ""
echo "1. MuseTalk weights (musetalkV15):"
echo "   https://huggingface.co/TMElyralab/MuseTalk/tree/main/models/musetalk"
echo ""
echo "2. DWPose model:"
echo "   https://huggingface.co/yzd-v/DWPose/resolve/main/dw-ll_ucoco_384.pth"
echo "   Save to: $MODELS_DIR/dwpose/dw-ll_ucoco_384.pth"
echo ""
echo "3. Face parsing model:"
echo "   https://huggingface.co/TMElyralab/MuseTalk/tree/main/models/face-parse-bisent"
echo ""
echo "4. SD-VAE model:"
echo "   https://huggingface.co/stabilityai/sd-vae-ft-mse"
echo ""
echo "5. Whisper model:"
echo "   https://huggingface.co/openai/whisper-tiny"
echo ""
echo ""
echo "Alternative: Use the MuseTalk download script directly:"
echo "  git clone https://github.com/TMElyralab/MuseTalk"
echo "  cd MuseTalk"
echo "  bash download_weights.sh"
echo "  cp -r models/* $MODELS_DIR/"
echo ""

# Verify structure
echo ""
echo "Current models directory structure:"
echo ""
find "$MODELS_DIR" -maxdepth 2 -type d | head -20

echo ""
echo "Done. Please ensure all model files are in place before running the server."
