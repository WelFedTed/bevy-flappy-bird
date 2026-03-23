#!/usr/bin/env bash
set -u

TEMP_DIR="./temp"
FORCE_DOWNLOAD=false

# Parse args
for arg in "$@"; do
  case "$arg" in
    --force-download)
      FORCE_DOWNLOAD=true
      ;;
  esac
done

SOURCES=(
    "https://archive.org/download/flappy-bird-v-1.2_202107/Flappy%20Bird%201.3.apk"
    "https://github.com/paulkr/Flappy-Bird/archive/refs/heads/master.zip"
)

APK="com.dotgears.flappybird-1.3-4-minAPI8.apk"
APK_PATH="$TEMP_DIR/$APK"
APK_EXPECTED_MD5="BF978C69C8E594E6FE301B677E69ACBC"

ZIP="paulkr_Flappy-Bird.zip"
ZIP_PATH="$TEMP_DIR/$ZIP"
ZIP_EXPECTED_MD5="19E22337C7DAFA9DD2B6522119ACDE1C"

DEST_DIR="./assets"

FILES_SOURCE1=(
  "assets/gfx/atlas.png"
  "assets/sounds/sfx_die.ogg"
  "assets/sounds/sfx_hit.ogg"
  "assets/sounds/sfx_point.ogg"
  "assets/sounds/sfx_swooshing.ogg"
  "assets/sounds/sfx_wing.ogg"
  "res/drawable/splash.png"
  "res/raw/atlas.txt"
)

FILES_SOURCE2=(
  "Flappy-Bird-master/lib/res/fonts/flappy-font.ttf"
  "Flappy-Bird-master/lib/res/img/icon.png"
)

OVERWRITE_ALL=false

require_command() {
  if ! command -v "$1" >/dev/null 2>&1; then
    echo "Error: required command '$1' not found."
    echo "Aborted"
    exit 1
  fi
}

calc_md5() {
  local file="$1"

  if command -v md5sum >/dev/null 2>&1; then
    md5sum "$file" | awk '{print toupper($1)}'
  elif command -v md5 >/dev/null 2>&1; then
    md5 -q "$file" | tr '[:lower:]' '[:upper:]'
  else
    echo "Error: No MD5 utility found (md5sum or md5)."
    echo "Aborted"
    exit 1
  fi
}

verify_md5() {
  local file="$1"
  local expected_md5="$2"
  local actual_md5

  actual_md5="$(calc_md5 "$file")"

  if [[ "$actual_md5" != "$expected_md5" ]]; then
    echo "File: $file"
    echo "Warning: MD5 hash mismatch."
    echo "Expected: $expected_md5"
    echo "Actual:   $actual_md5"
    read -r -p "Proceed anyway? (Y/N): " choice
    case "$choice" in
      y|Y) ;;
      *) echo "Aborted."; exit 1 ;;
    esac
  fi
}

download_if_needed() {
  local url="$1"
  local path="$2"
  local expected_md5="$3"

  if [[ -f "$path" && "$FORCE_DOWNLOAD" = false ]]; then
    echo "Found existing file: $path"
    local actual_md5
    actual_md5="$(calc_md5 "$path")"

    if [[ "$actual_md5" == "$expected_md5" ]]; then
      echo "MD5 valid, skipping download."
      return 0
    else
      echo "MD5 mismatch, re-downloading..."
    fi
  fi

  echo "Downloading $(basename "$path")..."
  curl -L "$url" -o "$path"
}

confirm_overwrite() {
  local dest_path="$1"

  if [[ -f "$dest_path" && "$OVERWRITE_ALL" = false ]]; then
    read -r -p "$dest_path exists. Overwrite? ([Y] Yes / [A] Yes to All / [N] No): " choice
    case "$choice" in
      y|Y) return 0 ;;
      a|A) OVERWRITE_ALL=true; return 0 ;;
      *) return 1 ;;
    esac
  fi

  return 0
}

extract_file() {
  local archive="$1"
  local inner_path="$2"

  local basename dest_path
  basename="$(basename "$inner_path")"
  dest_path="$DEST_DIR/$basename"

  if ! confirm_overwrite "$dest_path"; then
    echo "Skipping $basename"
    return 0
  fi

  echo "Extracting $inner_path -> $dest_path"
  if ! unzip -p "$archive" "$inner_path" > "$dest_path"; then
    echo "Failed to extract: $inner_path"
    rm -f "$dest_path"
    return 1
  fi
}

extract_group() {
  local archive="$1"
  shift
  local files=("$@")

  if [[ ! -f "$archive" ]]; then
    echo "Error: archive not found: $archive"
    echo "Aborted"
    exit 1
  fi

  for file in "${files[@]}"; do
    extract_file "$archive" "$file"
  done
}

require_command unzip
require_command curl

mkdir -p "$TEMP_DIR"

# Download with skip logic
download_if_needed "${SOURCES[0]}" "$APK_PATH" "$APK_EXPECTED_MD5"
download_if_needed "${SOURCES[1]}" "$ZIP_PATH" "$ZIP_EXPECTED_MD5"

# Verify after download
verify_md5 "$APK_PATH" "$APK_EXPECTED_MD5"
verify_md5 "$ZIP_PATH" "$ZIP_EXPECTED_MD5"

mkdir -p "$DEST_DIR"

echo "Extracting files from APK..."
extract_group "$APK_PATH" "${FILES_SOURCE1[@]}"

echo "Extracting files from ZIP..."
extract_group "$ZIP_PATH" "${FILES_SOURCE2[@]}"

echo "Done."
