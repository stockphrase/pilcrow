# Pilcrow

A bespoke encrypted journal for Linux. Entries are written in plain text, encrypted with a PGP key of your choosing, and saved as a standard Markdown file. Only you — with your private key — can read them.

![Icon](assets/pilcrow.svg)

---

## Features

- **Write** journal entries in a clean, distraction-free text area
- **Encrypt** each entry with a PGP public key from your GPG keyring before it ever touches disk
- **Decrypt** any PGP-encrypted block by pasting it into the Decrypt tab — the plaintext is held in memory only and wiped when cleared or the app is closed
- **Multiple journals** — each journal is a separate Markdown file with its own associated key
- Entries are stored in standard Markdown with ISO 8601 timestamps (`# 2026-04-09T14:32`), readable in any text editor (the bodies are ciphertext)
- Remembers your last opened journal across sessions

---

## Requirements

- Linux (Debian/Ubuntu/Pop!_OS or derivative)
- GPG installed and at least one key pair in your keyring

All other dependencies are handled automatically by the installer.

---

## Installation

```bash
git clone https://github.com/yourusername/pilcrow.git
cd pilcrow
chmod +x install.sh
./install.sh
```

The installer will:

1. Install system packages (`gnupg`, `python3`, `python3-tk`, `python3-venv`, `librsvg2-bin`) via `apt` if any are missing
2. Create a Python virtual environment at `~/.venvs/pgp_journal` and install `customtkinter` into it
3. Install a launcher script at `/usr/local/bin/pgp_journal`
4. Convert the SVG icon to PNG at all standard sizes and register it with your desktop environment
5. Install a `.desktop` entry so the app appears in your application launcher

After installing, search for **Pilcrow** in your app menu, or run `pilcrow` from a terminal.

---

## First run

You will need at least one GPG key pair. If you don't have one:

```bash
gpg --full-generate-key
```

On first launch, click **＋ New Journal** to choose a file location (`.md`) and select which public key to encrypt entries with. This choice is stored in the journal file header so the app can detect it automatically when you reopen the journal.

---

## Usage

### Write tab

Type your entry freely in the text area. Click **Encrypt & Save Entry** to encrypt and append it to the journal file. The write area is cleared after each save.

### Decrypt tab

Paste any `-----BEGIN PGP MESSAGE-----` block into the upper box and click **Decrypt**. GPG will prompt for your passphrase via your system's pinentry agent. The decrypted text appears in the lower box (read-only) and is never written to disk. It is wiped when you click **Clear All** or close the app.

---

## Journal file format

Journal files are plain Markdown. Each entry looks like this:

```
# 2026-04-09T14:32

-----BEGIN PGP MESSAGE-----

hQIMA7x2Kp9fT1QBAQ//Wd2mX8nLzP4oJkT9VqRs...
-----END PGP MESSAGE-----
```

You can store, sync, back up, or version-control your journal file freely — without the private key, the contents are unreadable.

---

## Uninstallation

```bash
./uninstall.sh
```

This removes the launcher, venv, icons, and desktop entry. Your journal `.md` files and GPG keys are never touched.

---

## License

MIT
