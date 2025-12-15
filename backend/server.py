#!/usr/bin/env python3
"""
MuseTalk HTTP Inference Server

Provides a REST API for lip-sync video generation using MuseTalk.
"""

import asyncio
import base64
import io
import logging
import os
import sys
import tempfile
import time
from pathlib import Path
from typing import Optional

import torch
from fastapi import FastAPI, HTTPException
from fastapi.middleware.cors import CORSMiddleware
from pydantic import BaseModel

# Add MuseTalk to path
sys.path.insert(0, str(Path(__file__).parent))

# Configure logging
logging.basicConfig(
    level=logging.INFO,
    format="%(asctime)s - %(name)s - %(levelname)s - %(message)s"
)
logger = logging.getLogger("musetalk-server")

# Server configuration
VERSION = "0.1.0"
HOST = os.environ.get("HOST", "0.0.0.0")
PORT = int(os.environ.get("PORT", 8000))
MODEL_VERSION = os.environ.get("MODEL_VERSION", "v1.5")

app = FastAPI(
    title="MuseTalk Inference Server",
    description="REST API for MuseTalk lip-sync generation",
    version=VERSION
)

# Enable CORS for CLI access
app.add_middleware(
    CORSMiddleware,
    allow_origins=["*"],
    allow_credentials=True,
    allow_methods=["*"],
    allow_headers=["*"],
)

# Global model state
model_loaded = False
inference_pipeline = None


class HealthResponse(BaseModel):
    status: str
    version: Optional[str] = None
    model_loaded: bool = False
    gpu_available: bool = False
    gpu_name: Optional[str] = None


class InferenceRequest(BaseModel):
    image: str  # Base64-encoded PNG
    audio: str  # Base64-encoded WAV
    fps: int = 25
    bbox_shift: int = 0  # Mouth openness adjustment


class FrameData(BaseModel):
    index: int
    data: str  # Base64-encoded PNG


class InferenceResponse(BaseModel):
    status: str
    total_frames: int
    fps: int
    frames: list[FrameData]
    processing_time_secs: float


def load_models():
    """Load MuseTalk models into GPU memory."""
    global model_loaded, inference_pipeline

    if model_loaded:
        return

    logger.info("Loading MuseTalk models...")
    start_time = time.time()

    try:
        # Import MuseTalk modules
        from musetalk.utils.utils import get_file_type, get_video_fps, datagen
        from musetalk.utils.preprocessing import get_landmark_and_bbox, read_imgs, coord_placeholder
        from musetalk.utils.blending import get_image_prepare_material, get_image_blending
        from musetalk.models.vae import VAE
        from musetalk.models.unet import UNet

        # Store pipeline components
        inference_pipeline = {
            "loaded": True,
            # Models will be loaded here during actual inference
        }

        model_loaded = True
        elapsed = time.time() - start_time
        logger.info(f"Models loaded in {elapsed:.2f}s")

    except Exception as e:
        logger.error(f"Failed to load models: {e}")
        raise


def run_inference(image_b64: str, audio_b64: str, fps: int, bbox_shift: int) -> tuple[list[str], float]:
    """
    Run MuseTalk inference.

    Returns list of base64-encoded PNG frames and processing time.
    """
    import cv2
    import numpy as np
    from PIL import Image

    start_time = time.time()

    # Decode input image
    image_bytes = base64.b64decode(image_b64)
    image = Image.open(io.BytesIO(image_bytes)).convert("RGB")
    image_np = np.array(image)

    # Decode input audio
    audio_bytes = base64.b64decode(audio_b64)

    with tempfile.TemporaryDirectory() as tmpdir:
        tmpdir = Path(tmpdir)

        # Save inputs to temp files
        image_path = tmpdir / "input.png"
        audio_path = tmpdir / "input.wav"
        output_dir = tmpdir / "output"
        output_dir.mkdir()

        image.save(image_path)
        with open(audio_path, "wb") as f:
            f.write(audio_bytes)

        # Run MuseTalk inference
        # Create inference config YAML for MuseTalk
        import yaml

        inference_config = {
            "task_0": {
                "video_path": str(image_path),
                "audio_path": str(audio_path),
            }
        }
        config_path = tmpdir / "inference_config.yaml"
        with open(config_path, "w") as f:
            yaml.dump(inference_config, f)

        try:
            from argparse import Namespace
            from scripts.inference import main as musetalk_inference

            # Create argparse Namespace with required arguments
            args = Namespace(
                ffmpeg_path="./ffmpeg-4.4-amd64-static/",
                gpu_id=0,
                vae_type="sd-vae",
                unet_config="./models/musetalk/config.json",
                unet_model_path="./models/musetalkV15/unet.pth" if MODEL_VERSION == "v1.5" else "./models/musetalk/unet.pth",
                whisper_dir="./models/whisper",
                inference_config=str(config_path),
                bbox_shift=bbox_shift,
                result_dir=str(output_dir),
                extra_margin=10,
                fps=fps,
                audio_padding_length_left=2,
                audio_padding_length_right=2,
                batch_size=8,
                output_vid_name=None,
                use_saved_coord=False,
                saved_coord=False,
                use_float16=True,
                parsing_mode="jaw",
                left_cheek_width=90,
                right_cheek_width=90,
                version="v15" if MODEL_VERSION == "v1.5" else "v1",
            )

            musetalk_inference(args)

        except Exception as e:
            # Fallback: use command-line inference
            import subprocess

            musetalk_dir = str(Path(__file__).parent)
            env = os.environ.copy()
            env["PYTHONPATH"] = musetalk_dir + ":" + env.get("PYTHONPATH", "")

            result = subprocess.run([
                "python", "scripts/inference.py",
                "--inference_config", str(config_path),
                "--result_dir", str(output_dir),
                "--fps", str(fps),
                "--bbox_shift", str(bbox_shift),
                "--use_float16",
            ], capture_output=True, text=True, cwd=musetalk_dir, env=env)

            if result.returncode != 0:
                raise RuntimeError(f"Inference failed: {result.stderr}\n{result.stdout}")

        # Collect output frames
        frames = []
        frame_files = sorted(output_dir.glob("*.png"))

        for frame_path in frame_files:
            with open(frame_path, "rb") as f:
                frame_b64 = base64.b64encode(f.read()).decode("utf-8")
                frames.append(frame_b64)

    processing_time = time.time() - start_time
    return frames, processing_time


@app.on_event("startup")
async def startup_event():
    """Load models on server startup."""
    logger.info(f"Starting MuseTalk server v{VERSION}")
    logger.info(f"CUDA available: {torch.cuda.is_available()}")
    if torch.cuda.is_available():
        logger.info(f"GPU: {torch.cuda.get_device_name(0)}")

    # Optionally preload models (can be slow, so disabled by default)
    if os.environ.get("PRELOAD_MODELS", "").lower() == "true":
        load_models()


@app.get("/health", response_model=HealthResponse)
async def health_check():
    """Health check endpoint."""
    gpu_available = torch.cuda.is_available()
    gpu_name = torch.cuda.get_device_name(0) if gpu_available else None

    return HealthResponse(
        status="healthy",
        version=VERSION,
        model_loaded=model_loaded,
        gpu_available=gpu_available,
        gpu_name=gpu_name
    )


@app.post("/infer", response_model=InferenceResponse)
async def infer(request: InferenceRequest):
    """
    Run lip-sync inference.

    Accepts base64-encoded image and audio, returns base64-encoded frames.
    """
    logger.info(f"Inference request: fps={request.fps}, bbox_shift={request.bbox_shift}")

    # Ensure models are loaded
    if not model_loaded:
        load_models()

    try:
        # Run inference in thread pool to avoid blocking
        loop = asyncio.get_event_loop()
        frames, processing_time = await loop.run_in_executor(
            None,
            run_inference,
            request.image,
            request.audio,
            request.fps,
            request.bbox_shift
        )

        logger.info(f"Generated {len(frames)} frames in {processing_time:.2f}s")

        return InferenceResponse(
            status="success",
            total_frames=len(frames),
            fps=request.fps,
            frames=[
                FrameData(index=i, data=frame)
                for i, frame in enumerate(frames)
            ],
            processing_time_secs=processing_time
        )

    except Exception as e:
        logger.error(f"Inference failed: {e}")
        raise HTTPException(status_code=500, detail=str(e))


@app.get("/")
async def root():
    """Root endpoint with API info."""
    return {
        "name": "MuseTalk Inference Server",
        "version": VERSION,
        "endpoints": {
            "/health": "Health check",
            "/infer": "POST - Run lip-sync inference"
        }
    }


if __name__ == "__main__":
    import uvicorn

    logger.info(f"Starting server on {HOST}:{PORT}")
    uvicorn.run(app, host=HOST, port=PORT)
