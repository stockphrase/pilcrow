#!/usr/bin/env bash
# Pilcrow — installer
# Installs from a pre-built binary if present, otherwise builds from source.
#
# Usage:
#   chmod +x install.sh && ./install.sh

set -e
SCRIPT_DIR="$(cd "$(dirname "${BASH_SOURCE[0]}")" && pwd)"

echo "=== Pilcrow Installer ==="
echo ""

echo "[1/3] Installing system dependencies..."

MISSING_PKGS=()
command -v gpg           &>/dev/null || MISSING_PKGS+=(gnupg)
command -v rsvg-convert  &>/dev/null || command -v inkscape &>/dev/null || MISSING_PKGS+=(librsvg2-bin)

if [ ! -f "$SCRIPT_DIR/pilcrow" ]; then
  pkg-config --exists gtk4 2>/dev/null || MISSING_PKGS+=(libgtk-4-dev)
  command -v pkg-config  &>/dev/null   || MISSING_PKGS+=(pkg-config)
  command -v cc          &>/dev/null   || MISSING_PKGS+=(build-essential)
else
  dpkg -s libgtk-4-1 &>/dev/null 2>&1 || MISSING_PKGS+=(libgtk-4-1)
fi

if [ ${#MISSING_PKGS[@]} -gt 0 ]; then
  echo "  Installing: ${MISSING_PKGS[*]}"
  sudo apt-get install -y "${MISSING_PKGS[@]}"
else
  echo "  ✓ All system dependencies present"
fi

echo "[2/3] Installing binary..."

if [ -f "$SCRIPT_DIR/pilcrow" ]; then
  echo "  Using pre-built binary"
  sudo install -m 755 "$SCRIPT_DIR/pilcrow" /usr/local/bin/pilcrow
else
  echo "  No pre-built binary found — building from source..."
  if ! command -v cargo &>/dev/null; then
    echo "  Rust not found — installing via rustup..."
    curl --proto '=https' --tlsv1.2 -sSf https://sh.rustup.rs | sh -s -- -y --no-modify-path
    source "$HOME/.cargo/env"
  fi
  export PATH="$HOME/.cargo/bin:$PATH"
  cd "$SCRIPT_DIR"
  cargo build --release
  sudo install -m 755 "$SCRIPT_DIR/target/release/pilcrow" /usr/local/bin/pilcrow
fi

echo "  ✓ Binary installed to /usr/local/bin/pilcrow"

echo "[3/3] Installing icons and desktop entry..."

SVG="$SCRIPT_DIR/assets/pilcrow.svg"
ICON_DIR="$HOME/.local/share/icons/hicolor"

if [ -f "$SVG" ]; then
  convert_icon() {
    local size=$1
    local dest="$ICON_DIR/${size}x${size}/apps"
    mkdir -p "$dest"
    if command -v rsvg-convert &>/dev/null; then
      rsvg-convert -w $size -h $size "$SVG" -o "$dest/pilcrow.png"
    elif command -v inkscape &>/dev/null; then
      inkscape "$SVG" --export-type=png \
        --export-width=$size --export-height=$size \
        --export-filename="$dest/pilcrow.png" 2>/dev/null
    fi
  }
  for size in 16 32 48 64 128 256 512; do
    convert_icon $size && echo "  ✓ Icon ${size}x${size}"
  done
  gtk-update-icon-cache -f -t "$ICON_DIR" 2>/dev/null || true
else
  echo "  Warning: assets/pilcrow.svg not found, skipping icons."
fi

DESKTOP_DIR="$HOME/.local/share/applications"
mkdir -p "$DESKTOP_DIR"
cat > "$DESKTOP_DIR/pilcrow.desktop" <<EOF
[Desktop Entry]
Version=1.0
Type=Application
Name=Pilcrow
GenericName=Encrypted Journal
Comment=Write and read encrypted journal entries using your PGP keys
Exec=/usr/local/bin/pilcrow
Icon=pilcrow
Terminal=false
Categories=Office;Utility;Security;
Keywords=journal;diary;pgp;gpg;encrypt;privacy;
StartupNotify=true
StartupWMClass=pilcrow
EOF
chmod 644 "$DESKTOP_DIR/pilcrow.desktop"
update-desktop-database "$DESKTOP_DIR" 2>/dev/null || true
echo "  ✓ Desktop entry installed"

echo ""
echo "=== Installation complete ==="
echo ""
echo "You can now:"
echo "  • Launch from your app menu (search 'Pilcrow')"
echo "  • Run from terminal: pilcrow"
echo ""
echo "Note: you will need at least one GPG key pair in your keyring."
echo "To generate one:  gpg --full-generate-key"
