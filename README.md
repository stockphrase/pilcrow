# Pilcrow

A bespoke encrypted journal for Linux. Entries are written in plain text, encrypted with a PGP key of your choosing, and saved as a standard Markdown file. Only you — with your private key — can read them.

![Icon](assets/pilcrow.svg)

---

## Features

- **Write** journal entries in a clean, distraction-free text area
- **Encrypt** each entry with a PGP public key from your GPG keyring before it ever touches disk
- **Decrypt Journal** tab decrypts all entries at once into a scrollable, searchable view — plaintext is held in memory only and wiped on clear or close
- **Search** decrypted entries with Ctrl+F — highlights all matches with ▲/▼ navigation
- **Multiple journals** — each journal is a separate Markdown file with its own associated key
- Entries are stored in standard Markdown with ISO 8601 timestamps (`# 2026-04-09T14:32`), readable in any text editor (the bodies are ciphertext)
- Remembers your last opened journal across sessions
- Written in Rust with GTK4 — sensitive plaintext is zeroed from memory on close

---

## Requirements

- Linux (Debian/Ubuntu/Pop!_OS or derivative)
- GPG installed with at least one key pair in your keyring
- `libgtk-4-1` (installed automatically by the installer)

---

## Installation

### Option A — pre-built binary (recommended)

Download the latest release tarball from the [releases page](https://github.com/stockphrase/pilcrow/releases):

```bash
tar -xzf pilcrow-v0.1.0-linux-x86_64.tar.gz
cd pilcrow-release
chmod +x install.sh && ./install.sh
```

No Rust toolchain required. The installer will handle all system dependencies via `apt`.

### Option B — build from source

```bash
git clone https://github.com/stockphrase/pilcrow.git
cd pilcrow
chmod +x install.sh && ./install.sh
```

The installer will set up the Rust toolchain automatically if it isn't present. The first build may take a few minutes while Cargo compiles dependencies.

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

Type your entry freely in the text area. Click **🔒 Encrypt & Save Entry** to encrypt and append it to the journal file. The write area is cleared after each save.

### Decrypt Journal tab

Click **🔓 Decrypt All Entries** — GPG will prompt for your passphrase once via your system's pinentry agent, then decrypt all entries in sequence. The decrypted journal appears in a scrollable read-only view.

Press **Ctrl+F** to search — all matches are highlighted as you type, with ▲/▼ buttons to step through them.

Click **Clear** or close the app to wipe the decrypted text from memory. The GPG agent cache is also cleared on close.

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

## GPG passphrase caching

By default the GPG agent caches your passphrase for 10 minutes. To tighten this, add to `~/.gnupg/gpg-agent.conf`:

```
default-cache-ttl 60
max-cache-ttl 120
```

Then reload the agent:

```bash
gpg-connect-agent reloadagent /bye
```

---

## Uninstallation

```bash
./uninstall.sh
```

This removes the binary, icons, and desktop entry. Your journal `.md` files and GPG keys are never touched.

---

## License

MIT# Pilcrow

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

### Option A — pre-built binary (recommended)

Download the latest release tarball from the [releases page](https://github.com/yourusername/pilcrow/releases):

```bash
tar -xzf pilcrow-v0.1.0-linux-x86_64.tar.gz
cd pilcrow-release
chmod +x install.sh && ./install.sh
```

Only `gnupg`, `libgtk-4-1`, and `librsvg2-bin` are required — no Rust toolchain needed.

### Option B — build from source

```bash
git clone https://github.com/yourusername/pilcrow.git
cd pilcrow
chmod +x install.sh && ./install.sh
```

The installer will set up the full Rust toolchain automatically if it isn't present.

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
