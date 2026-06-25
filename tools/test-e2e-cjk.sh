#!/usr/bin/env bash
# CJK end-to-end test runner.
#
# Usage: ./tools/test-e2e-cjk.sh [model_dir]
#
# model_dir defaults to ./models and must contain:
#   PP-OCRv5_server_rec_infer.onnx
#   alphabet.txt
#
# Test images and expected outputs live in ocrs-cli/test-data/cjk/.
# Each <name>.png must have a matching <name>.expected.txt.
# Exit code is 0 if all tests pass, 1 otherwise.

set -euo pipefail

MODEL_DIR="${1:-models}"
REC_MODEL="$MODEL_DIR/PP-OCRv5_server_rec_infer.onnx"
ALPHABET="$MODEL_DIR/alphabet.txt"
TEST_DIR="ocrs-cli/test-data/cjk"
BINARY="target/release/ocrs"

# Validate prerequisites
if [ ! -f "$REC_MODEL" ]; then
    echo "ERROR: recognition model not found at $REC_MODEL"
    echo "  Download PP-OCRv5_server_rec_infer.onnx to $MODEL_DIR/ (see README for instructions)"
    exit 1
fi
if [ ! -f "$ALPHABET" ]; then
    echo "ERROR: alphabet file not found at $ALPHABET"
    echo "  Extract alphabet from the model YAML config (see README for instructions)"
    exit 1
fi

# Build if necessary
if [ ! -f "$BINARY" ]; then
    echo "Building release binary with ONNX support..."
    cargo build --release -p ocrs-cli --features onnx
fi

PASS=0
FAIL=0
SKIP=0

for img in "$TEST_DIR"/*.png; do
    base="${img%.png}"
    name="$(basename "$base")"
    expected_txt="$base.expected.txt"

    if [ ! -f "$expected_txt" ]; then
        echo "SKIP $name (no .expected.txt)"
        SKIP=$((SKIP + 1))
        continue
    fi

    actual=$("$BINARY" \
        --rec-model "$REC_MODEL" \
        --alphabet-file "$ALPHABET" \
        "$img" 2>/dev/null || true)

    expected="$(cat "$expected_txt")"

    if [ "$actual" = "$expected" ]; then
        echo "PASS $name"
        PASS=$((PASS + 1))
    else
        echo "FAIL $name"
        echo "  expected: $(printf '%q' "$expected")"
        echo "  actual:   $(printf '%q' "$actual")"
        FAIL=$((FAIL + 1))
    fi
done

# --- PDF searchable output E2E test ---
TEST_PDF="$TEST_DIR/test_ja.pdf"
EXPECTED_PDF_TXT="$TEST_DIR/test_ja_searchable.expected.txt"

if [ ! -f "$TEST_PDF" ] || [ ! -f "$EXPECTED_PDF_TXT" ]; then
    echo "SKIP PDF searchable (test_ja.pdf or expected file not found)"
    SKIP=$((SKIP + 1))
elif ! command -v pdftotext >/dev/null 2>&1; then
    echo "SKIP PDF searchable (pdftotext not available)"
    SKIP=$((SKIP + 1))
else
    OUT_PDF="/tmp/ocrs_test_searchable_$$.pdf"
    "$BINARY" \
        --rec-model "$REC_MODEL" \
        --alphabet-file "$ALPHABET" \
        "$TEST_PDF" \
        --output-pdf "$OUT_PDF" >/dev/null 2>&1 || true

    extracted="$(pdftotext "$OUT_PDF" - 2>/dev/null | tr -d '\n\f')"
    expected_pdf="$(cat "$EXPECTED_PDF_TXT" | tr -d '\n')"
    rm -f "$OUT_PDF"

    if [ "$extracted" = "$expected_pdf" ]; then
        echo "PASS PDF searchable (test_ja.pdf → pdftotext)"
        PASS=$((PASS + 1))
    else
        echo "FAIL PDF searchable"
        echo "  expected: $(printf '%q' "$expected_pdf")"
        echo "  actual:   $(printf '%q' "$extracted")"
        FAIL=$((FAIL + 1))
    fi
fi

echo ""
echo "CJK E2E results: $PASS passed, $FAIL failed, $SKIP skipped"
[ "$FAIL" -eq 0 ]
