#!/usr/bin/env bash

APK="com.dotgears.flappybird-1.3-4-minAPI8.apk"
EXPECTED_MD5="BF978C69C8E594E6FE301B677E69ACBC"
DEST_DIR="./assets"

FILES=(
"assets/gfx/atlas.png"
"assets/sounds/sfx_die.ogg"
"assets/sounds/sfx_hit.ogg"
"assets/sounds/sfx_point.ogg"
"assets/sounds/sfx_swooshing.ogg"
"assets/sounds/sfx_wing.ogg"
"res/drawable/splash.png"
"res/drawable-xxxhdpi/ic_launcher.png"
"res/raw/atlas.txt"
)

OVERWRITE_ALL=false

# Check if APK exists
if [[ ! -f "$APK" ]]; then
    echo "Error: $APK not found in current directory."
    exit 1
fi

# Calculate MD5
if command -v md5sum >/dev/null 2>&1; then
    ACTUAL_MD5=$(md5sum "$APK" | awk '{print toupper($1)}')
elif command -v md5 >/dev/null 2>&1; then
    ACTUAL_MD5=$(md5 -q "$APK" | tr '[:lower:]' '[:upper:]')
else
    echo "Error: No MD5 utility found (md5sum or md5)."
    exit 1
fi

# Verify MD5
if [[ "$ACTUAL_MD5" != "$EXPECTED_MD5" ]]; then
    echo "File: $APK"
    echo "Warning: MD5 hash mismatch."
    echo "Expected: $EXPECTED_MD5"
    echo "Actual:   $ACTUAL_MD5"
    read -p "Proceed anyway? (Y/N): " choice
    case "$choice" in
        y|Y) ;;
        *) echo "Aborted."; exit 1 ;;
    esac
fi

mkdir -p "$DEST_DIR"

for FILE in "${FILES[@]}"; do
    BASENAME=$(basename "$FILE")
    DEST_PATH="$DEST_DIR/$BASENAME"

    if [[ -f "$DEST_PATH" && "$OVERWRITE_ALL" = false ]]; then
        read -p "$DEST_PATH exists. Overwrite? ([Y] Yes [A] Yes to All / [N] No): " choice
        case "$choice" in
            y|Y) ;;
            a|A)
                OVERWRITE_ALL=true
                ;;
            *)
                echo "Skipping $BASENAME"
                continue
                ;;
        esac
    fi

    echo "Extracting $BASENAME..."
    unzip -p "$APK" "$FILE" > "$DEST_PATH"

    if [[ $? -ne 0 ]]; then
        echo "Failed to extract $FILE"
    fi
done

echo "Done."
