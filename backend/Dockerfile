# MuseTalk Inference Server
# For NVIDIA GPUs with CUDA support

FROM nvidia/cuda:11.8.0-cudnn8-devel-ubuntu22.04

# Prevent interactive prompts during package installation
ENV DEBIAN_FRONTEND=noninteractive
ENV TZ=UTC

# Install system dependencies
RUN apt-get update && apt-get install -y \
    python3.10 \
    python3.10-venv \
    python3-pip \
    git \
    wget \
    ffmpeg \
    libgl1-mesa-glx \
    libglib2.0-0 \
    libsm6 \
    libxext6 \
    libxrender-dev \
    && rm -rf /var/lib/apt/lists/*

# Set python3.10 as default
RUN update-alternatives --install /usr/bin/python python /usr/bin/python3.10 1 \
    && update-alternatives --install /usr/bin/python3 python3 /usr/bin/python3.10 1

# Upgrade pip
RUN python -m pip install --upgrade pip

# Set working directory
WORKDIR /app

# Clone MuseTalk repository
RUN git clone https://github.com/TMElyralab/MuseTalk.git /app/musetalk

WORKDIR /app/musetalk

# Install PyTorch with CUDA 11.8 support
RUN pip install torch==2.0.1 torchvision==0.15.2 torchaudio==2.0.2 \
    --index-url https://download.pytorch.org/whl/cu118

# Install chumpy first (has broken setup.py that needs --no-build-isolation)
RUN pip install chumpy --no-build-isolation

# Install MMlab packages via openmim (pin mmcv<2.2.0 for compatibility)
RUN pip install openmim \
    && mim install mmengine \
    && mim install "mmcv>=2.0.1,<2.2.0" \
    && mim install "mmdet>=3.1.0" \
    && mim install "mmpose>=1.1.0"

# Install MuseTalk requirements
RUN pip install -r requirements.txt

# Install additional server dependencies
RUN pip install fastapi uvicorn python-multipart aiofiles

# Copy our server wrapper
COPY server.py /app/musetalk/server.py

# Create directories for models (to be mounted or downloaded)
RUN mkdir -p /app/musetalk/models

# Expose the API port
EXPOSE 8000

# Health check
HEALTHCHECK --interval=30s --timeout=10s --start-period=60s --retries=3 \
    CMD curl -f http://localhost:8000/health || exit 1

# Default command - run the inference server
CMD ["python", "server.py"]
