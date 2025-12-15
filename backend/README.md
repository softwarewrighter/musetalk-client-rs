# MuseTalk Backend Server

Docker-based MuseTalk inference server for lip-sync video generation.

## Requirements

- Docker with NVIDIA GPU support
- NVIDIA GPU with CUDA capability (RTX 3060 or better recommended)
- nvidia-container-toolkit installed
- ~10GB disk space for Docker image
- ~5GB disk space for model weights

## Quick Start

```bash
# 1. Build the Docker image
./build.sh

# 2. Download model weights
./download_models.sh

# 3. Start the server
./run.sh --detach

# 4. Check server status
curl http://localhost:8000/health
```

## Scripts

| Script | Description |
|--------|-------------|
| `build.sh` | Build the Docker image |
| `download_models.sh` | Download model weights from HuggingFace |
| `run.sh` | Start the inference server |
| `stop.sh` | Stop the inference server |

## Configuration

### Environment Variables

| Variable | Default | Description |
|----------|---------|-------------|
| `MUSETALK_MODELS_DIR` | `./models` | Path to model weights |
| `MUSETALK_PORT` | `8000` | Server port |

### run.sh Options

```
./run.sh [OPTIONS]

Options:
  -d, --detach     Run in background
  -p, --port PORT  Server port (default: 8000)
  --models DIR     Path to models directory
  -h, --help       Show help
```

## Model Weights

Models must be downloaded before running the server. The expected structure:

```
models/
  musetalk/           # or musetalkV15/
    ...
  dwpose/
    dw-ll_ucoco_384.pth
  face-parse-bisent/
    79999_iter.pth
    resnet18-5c106cde.pth
  sd-vae/
    config.json
    diffusion_pytorch_model.safetensors
  whisper/
    ...
```

### Download Options

1. **Use download script** (recommended):
   ```bash
   ./download_models.sh
   ```

2. **Clone MuseTalk repo and copy models**:
   ```bash
   git clone https://github.com/TMElyralab/MuseTalk
   cd MuseTalk
   bash download_weights.sh
   cp -r models/* /path/to/backend/models/
   ```

3. **Manual download** from HuggingFace:
   - https://huggingface.co/TMElyralab/MuseTalk

## API Endpoints

### Health Check

```
GET /health

Response:
{
  "status": "healthy",
  "version": "0.1.0",
  "model_loaded": true,
  "gpu_available": true,
  "gpu_name": "NVIDIA GeForce RTX 3060"
}
```

### Inference

```
POST /infer
Content-Type: application/json

{
  "image": "<base64-encoded PNG>",
  "audio": "<base64-encoded WAV>",
  "fps": 25,
  "bbox_shift": 0
}

Response:
{
  "status": "success",
  "total_frames": 250,
  "fps": 25,
  "frames": [
    {"index": 0, "data": "<base64-encoded PNG>"},
    {"index": 1, "data": "<base64-encoded PNG>"},
    ...
  ],
  "processing_time_secs": 12.5
}
```

## GPU Memory Requirements

| GPU VRAM | Batch Size | Notes |
|----------|------------|-------|
| 4GB | 1 | Minimum, may be slow |
| 8GB | 2-4 | Good performance |
| 12GB+ | 4-8 | Best performance |

The RTX 3060 (12GB) should handle inference comfortably.

## Troubleshooting

### CUDA not available

Ensure nvidia-container-toolkit is installed:
```bash
# Arch Linux
sudo pacman -S nvidia-container-toolkit
sudo systemctl restart docker
```

### Out of memory

Reduce batch size or use fp16 precision (enabled by default).

### Model loading fails

Check that all model files are present in the models directory.
Run `./download_models.sh` to verify the structure.

## Network Access

By default, the server binds to `0.0.0.0:8000`, accessible from other machines.

To access from the CLI client on another machine:
```bash
musetalk-cli --server http://big72:8000 -i image.png -a audio.wav -o output.mp4
```

## Logs

View server logs:
```bash
docker logs -f musetalk
```

## Development

To run the server outside Docker for development:

```bash
cd /path/to/MuseTalk
pip install -r requirements.txt
pip install fastapi uvicorn
python server.py
```
