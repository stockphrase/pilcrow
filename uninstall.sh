#!/usr/bin/env bash
# Pilcrow — uninstaller
#
# Usage:
#   chmod +x uninstall.sh && ./uninstall.sh

set -e

echo "=== PGP Journal Uninstaller ==="
echo ""

# ── Launcher ──────────────────────────────────────────────────────────────────
if [ -f /usr/local/bin/pgp_journal ]; then
  sudo rm /usr/local/bin/pgp_journal
  echo "✓ Removed launcher from /usr/local/bin"
else
  echo "  Launcher not found, skipping"
fi

# ── Icons ─────────────────────────────────────────────────────────────────────
ICON_DIR="$HOME/.local/share/icons/hicolor"
FOUND_ICONS=0
for size in 16 32 48 64 128 256 512; do
  ICON="$ICON_DIR/${size}x${size}/apps/pilcrow.png"
  if [ -f "$ICON" ]; then
    rm "$ICON"
    FOUND_ICONS=1
  fi
done
if [ $FOUND_ICONS -eq 1 ]; then
  gtk-update-icon-cache -f -t "$ICON_DIR" 2>/dev/null || true
  echo "✓ Removed icons"
else
  echo "  Icons not found, skipping"
fi

# ── Desktop entry ─────────────────────────────────────────────────────────────
DESKTOP="$HOME/.local/share/applications/pilcrow.desktop"
if [ -f "$DESKTOP" ]; then
  rm "$DESKTOP"
  update-desktop-database "$HOME/.local/share/applications" 2>/dev/null || true
  echo "✓ Removed desktop entry"
else
  echo "  Desktop entry not found, skipping"
fi

# ── Config ────────────────────────────────────────────────────────────────────
CONFIG="$HOME/.pilcrow_config.json"
if [ -f "$CONFIG" ]; then
  read -r -p "Remove saved config (~/.pilcrow_config.json)? [y/N] " confirm
  if [[ "$confirm" =~ ^[Yy]$ ]]; then
    rm "$CONFIG"
    echo "✓ Removed config file"
  else
    echo "  Config file kept"
  fi
fi

echo ""
echo "=== Pilcrow uninstall complete ==="
echo ""
echo "Your journal .md files have not been touched."