#!/usr/bin/env bash
set -u

TEMP_DIR="./temp"
DEST_DIR="./assets"

FORCE_DOWNLOAD=false
CLEAN=false
OVERWRITE_ALL=false

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

FILES_SOURCE1=(
  "assets/gfx/atlas.png"
  "assets/sounds/sfx_die.ogg"
  "assets/sounds/sfx_hit.ogg"
  "assets/sounds/sfx_point.ogg"
  "assets/sounds/sfx_swooshing.ogg"
  "assets/sounds/sfx_wing.ogg"
  "res/drawable/splash.png"
  # "res/drawable-xxxhdpi/ic_launcher.png"
  "res/raw/atlas.txt"
)

FILES_SOURCE2=(
  "Flappy-Bird-master/lib/res/fonts/flappy-font.ttf"
  "Flappy-Bird-master/lib/res/img/icon.png"
)

log() {
  printf '[INFO] %s\n' "$*"
}

warn() {
  printf '[WARN] %s\n' "$*"
}

error() {
  printf '[ERROR] %s\n' "$*" >&2
}

abort() {
  error "$*"
  echo "Aborted"
  exit 1
}

usage() {
  cat <<EOF
Sources Download Script
This script downloads and extracts the source assets required to build the Bevy Flappy Bird game.

Usage: $0 [options]

Options:
  --force-download   Re-download source archives even if valid cached copies exist
  --clean            Delete ./temp after successful extraction
  --overwrite        Overwrite all extracted files without prompting
  --help             Show this help message
EOF
}

for arg in "$@"; do
  case "$arg" in
    --force-download)
      FORCE_DOWNLOAD=true
      ;;
    --clean)
      CLEAN=true
      ;;
    --overwrite)
      OVERWRITE_ALL=true
      ;;
    --help|-h)
      usage
      exit 0
      ;;
    *)
      abort "Unknown argument: $arg"
      ;;
  esac
done

require_command() {
  if ! command -v "$1" >/dev/null 2>&1; then
    abort "required command '$1' not found."
  fi
}

calc_md5() {
  local file="$1"

  if command -v md5sum >/dev/null 2>&1; then
    md5sum "$file" | awk '{print toupper($1)}'
  elif command -v md5 >/dev/null 2>&1; then
    md5 -q "$file" | tr '[:lower:]' '[:upper:]'
  else
    abort "No MD5 utility found (md5sum or md5)."
  fi
}

verify_md5() {
  local file="$1"
  local expected_md5="$2"
  local actual_md5

  actual_md5="$(calc_md5 "$file")"

  if [[ "$actual_md5" != "$expected_md5" ]]; then
    echo "File: $file"
    warn "MD5 hash mismatch."
    echo "Expected: $expected_md5"
    echo "Actual:   $actual_md5"
    read -r -p "Proceed anyway? (Y/N): " choice
    case "$choice" in
      y|Y) ;;
      *) abort "User chose not to proceed." ;;
    esac
  fi
}

download_file() {
  local url="$1"
  local path="$2"
  local expected_md5="$3"

  if [[ -f "$path" && "$FORCE_DOWNLOAD" = false ]]; then
    local actual_md5
    actual_md5="$(calc_md5 "$path")"

    if [[ "$actual_md5" == "$expected_md5" ]]; then
      log "Using cached file: $path"
      return 0
    fi

    warn "Cached file failed MD5, re-downloading: $path"
    rm -f "$path"
  fi

  log "Downloading $(basename "$path")"
  if ! curl -# -L "$url" -o "$path"; then
    rm -f "$path"
    return 1
  fi
}

confirm_overwrite() {
  local dest_path="$1"

  if [[ -f "$dest_path" && "$OVERWRITE_ALL" = false ]]; then
    read -r -p "$dest_path exists. Overwrite? ([Y] Yes / [A] Yes to All / [N] No): " choice
    case "$choice" in
      y|Y)
        return 0
        ;;
      a|A)
        OVERWRITE_ALL=true
        return 0
        ;;
      *)
        return 1
        ;;
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
    log "Skipping $basename"
    return 0
  fi

  log "Extracting $inner_path -> $dest_path"
  if ! unzip -p "$archive" "$inner_path" > "$dest_path"; then
    error "Failed to extract: $inner_path"
    rm -f "$dest_path"
    return 1
  fi
}

extract_group() {
  local archive="$1"
  shift
  local files=("$@")

  [[ -f "$archive" ]] || abort "archive not found: $archive"

  local file
  for file in "${files[@]}"; do
    extract_file "$archive" "$file"
  done
}

cleanup_temp() {
  if [[ "$CLEAN" = true ]]; then
    log "Cleaning temp directory: $TEMP_DIR"
    rm -rf "$TEMP_DIR"
  fi
}

require_command unzip
require_command curl

mkdir -p "$TEMP_DIR"
mkdir -p "$DEST_DIR"

log "Preparing source archives"

download_file "${SOURCES[0]}" "$APK_PATH" "$APK_EXPECTED_MD5" \
  || abort "Download failed: $APK"

download_file "${SOURCES[1]}" "$ZIP_PATH" "$ZIP_EXPECTED_MD5" \
  || abort "Download failed: $ZIP"

[[ -f "$APK_PATH" ]] || abort "Missing file after download: $APK_PATH"
[[ -f "$ZIP_PATH" ]] || abort "Missing file after download: $ZIP_PATH"

log "Verifying archive checksums"
verify_md5 "$APK_PATH" "$APK_EXPECTED_MD5"
verify_md5 "$ZIP_PATH" "$ZIP_EXPECTED_MD5"

log "Extracting files from APK"
extract_group "$APK_PATH" "${FILES_SOURCE1[@]}"

log "Extracting files from ZIP"
extract_group "$ZIP_PATH" "${FILES_SOURCE2[@]}"

cleanup_temp

log "Done."
