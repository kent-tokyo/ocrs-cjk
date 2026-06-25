#!/usr/bin/env python3
"""Download PP-OCRv5 recognition model files from Hugging Face.

Usage: python3 tools/download-models.py [models_dir]

Downloads:
  PP-OCRv5_server_rec_infer.onnx  (~84 MB ONNX recognition model)
  PP-OCRv5_server_rec_infer.yml   (model config containing character dict)

Requires: pip install huggingface_hub
"""
import sys

from huggingface_hub import hf_hub_download

models_dir = sys.argv[1] if len(sys.argv) > 1 else "models"
repo = "marsena/paddleocr-onnx-models"

for filename in ("PP-OCRv5_server_rec_infer.onnx", "PP-OCRv5_server_rec_infer.yml"):
    print(f"Downloading {filename}...")
    hf_hub_download(repo_id=repo, filename=filename, local_dir=models_dir)
    print(f"  -> {models_dir}/{filename}")

print("Done.")
