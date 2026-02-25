#!/usr/bin/env bash
set -euo pipefail

ROOT_DIR="$(cd "$(dirname "$0")/.." && pwd)"
OUT_DIR_CURATED="$ROOT_DIR/showcase/videos_curated"
OUT_DIR_LEGACY="$ROOT_DIR/showcase/videos"
DO_FRESH=0

if [[ "${1:-}" == "--fresh" ]]; then
  DO_FRESH=1
fi

mkdir -p "$OUT_DIR_CURATED" "$OUT_DIR_LEGACY"

echo "[showcase] root: $ROOT_DIR"
echo "[showcase] curated output: $OUT_DIR_CURATED"
echo "[showcase] legacy output:  $OUT_DIR_LEGACY"

if [[ "$DO_FRESH" == "1" ]]; then
  echo "[showcase] Optional fresh renders (best effort)"
  if command -v cargo >/dev/null 2>&1; then
    (
      cd "$ROOT_DIR"
      cargo run -p vidra-cli -- render issues/001-anchor-point-mismatch/proof/test.vidra --output "$OUT_DIR_CURATED/anchor_centered_fresh.mp4" || true
    )
  else
    echo "[showcase] cargo not found; skipping fresh render"
  fi
else
  echo "[showcase] skipping fresh render (use --fresh to enable)"
fi

echo "[showcase] Collecting curated videos"
copy_if_exists() {
  local src="$1"
  local dest="$2"
  if [[ -f "$src" ]]; then
    cp -f "$src" "$dest"
    echo "  + $dest"
  else
    echo "  - missing: $src"
  fi
}

copy_if_exists "$ROOT_DIR/issues/001-anchor-point-mismatch/proof/result.mp4" "$OUT_DIR_CURATED/anchor_centered.mp4"
copy_if_exists "$ROOT_DIR/issues/003-audio-muxing-silent/proof/result.mp4" "$OUT_DIR_CURATED/audio_muxing_verified.mp4"
copy_if_exists "$ROOT_DIR/issues/002-strict-image-decoding/proof/result_fixed_text.mp4" "$OUT_DIR_CURATED/image_decode_fixed.mp4"
copy_if_exists "$ROOT_DIR/output/test_app.mp4" "$OUT_DIR_CURATED/app_flow.mp4"
copy_if_exists "$ROOT_DIR/output/test_brand.mp4" "$OUT_DIR_CURATED/brand_flow.mp4"
copy_if_exists "$ROOT_DIR/output/multi_target_test_16x9.mp4" "$OUT_DIR_CURATED/multi_target_16x9.mp4"
copy_if_exists "$ROOT_DIR/output/multi_target_test_9x16.mp4" "$OUT_DIR_CURATED/multi_target_9x16.mp4"

echo "[showcase] Collecting legacy videos"
copy_if_exists "$ROOT_DIR/output/demo.mp4" "$OUT_DIR_LEGACY/demo.mp4"
copy_if_exists "$ROOT_DIR/benchmark/bench_1080p.mp4" "$OUT_DIR_LEGACY/bench_1080p.mp4"
copy_if_exists "$ROOT_DIR/benchmark/bench_4k.mp4" "$OUT_DIR_LEGACY/bench_4k.mp4"

if command -v ffprobe >/dev/null 2>&1; then
  AUDIO_FILE="$OUT_DIR_CURATED/audio_muxing_verified.mp4"
  if [[ -f "$AUDIO_FILE" ]]; then
    audio_codec="$(ffprobe -v error -select_streams a -show_entries stream=codec_name -of default=nk=1:nw=1 "$AUDIO_FILE" | head -n 1 || true)"
    if [[ -n "$audio_codec" ]]; then
      echo "[showcase] audio check: OK ($audio_codec) in $AUDIO_FILE"
    else
      echo "[showcase] audio check: MISSING audio stream in $AUDIO_FILE"
    fi
  fi
fi

echo "[showcase] Done"
echo "[showcase] Curated set"
ls -1 "$OUT_DIR_CURATED"
echo "[showcase] Legacy set"
ls -1 "$OUT_DIR_LEGACY"
