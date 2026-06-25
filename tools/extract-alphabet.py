#!/usr/bin/env python3
"""Extract the character dictionary from a PP-OCRv5 YAML config file.

Usage: python3 tools/extract-alphabet.py [models_dir]

Reads {models_dir}/PP-OCRv5_server_rec_infer.yml and writes the
concatenated character list to {models_dir}/alphabet.txt, which is the
format expected by --alphabet-file in the ocrs CLI.
"""
import sys

import yaml

models_dir = sys.argv[1] if len(sys.argv) > 1 else "models"
cfg_path = f"{models_dir}/PP-OCRv5_server_rec_infer.yml"
out_path = f"{models_dir}/alphabet.txt"

with open(cfg_path, encoding="utf-8") as f:
    cfg = yaml.safe_load(f)

chars = cfg["PostProcess"]["character_dict"]

# Some entries (e.g. country-flag emoji) span two Unicode code points.
# ocrs maps one label → one char, so collapse each entry to its first code point.
fixed = [c[0] if len(c) > 1 else c for c in chars]

# PaddleOCR appends a space label when use_space_char=True (the default).
fixed.append(" ")

with open(out_path, "w", encoding="utf-8") as f:
    f.write("".join(fixed))

print(f"Written {len(fixed)} characters → {out_path}")
