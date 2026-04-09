use crate::{config, gpg};
use chrono::Local;
use gtk::prelude::*;
use gtk::{
    glib, Application, ApplicationWindow, Box as GBox, Button, Dialog, Entry,
    FileChooserAction, FileChooserDialog, Label, Notebook, ResponseType,
    ScrolledWindow, TextView, TextBuffer, DropDown, StringList,
    Orientation, Align, WrapMode, Image,
};
use std::cell::RefCell;
use std::fs::{self, OpenOptions};
use std::io::Write;
use std::rc::Rc;
use std::sync::{Arc, Mutex};
use zeroize::Zeroizing;

#[derive(Default)]
struct JournalState {
    path: Option<String>,
    key_id: Option<String>,
    key_uid: Option<String>,
}

pub fn build_ui(app: &Application) {
    let state = Rc::new(RefCell::new(JournalState::default()));

    // ── Window ────────────────────────────────────────────────────────────────
    let window = ApplicationWindow::builder()
        .application(app)
        .title("PGP Journal")
        .default_width(860)
        .default_height(680)
        .build();

    let root = GBox::new(Orientation::Vertical, 0);

    // ── Top bar ───────────────────────────────────────────────────────────────
    let topbar = GBox::new(Orientation::Horizontal, 8);
    topbar.set_margin_start(12);
    topbar.set_margin_end(12);
    topbar.set_margin_top(8);
    topbar.set_margin_bottom(8);

    // App icon from installed hicolor theme, fallback to named icon
    let app_icon = Image::from_icon_name("pilcrow");
    app_icon.set_pixel_size(32);
    topbar.append(&app_icon);

    let btn_new = Button::with_label("＋ New Journal");
    let btn_open = Button::with_label("📂 Open Journal");
    let status_label = Label::new(Some("No journal open"));
    status_label.set_halign(Align::End);
    status_label.set_hexpand(true);

    topbar.append(&btn_new);
    topbar.append(&btn_open);
    topbar.append(&status_label);
    root.append(&topbar);

    // ── Notebook (tabs) ───────────────────────────────────────────────────────
    let notebook = Notebook::new();
    notebook.set_vexpand(true);

    // ── Write tab ─────────────────────────────────────────────────────────────
    let write_vbox = GBox::new(Orientation::Vertical, 8);
    write_vbox.set_margin_start(12);
    write_vbox.set_margin_end(12);
    write_vbox.set_margin_top(8);
    write_vbox.set_margin_bottom(8);

    let meta_row = GBox::new(Orientation::Horizontal, 0);
    let key_label = Label::new(Some("Key: —"));
    key_label.set_halign(Align::Start);
    key_label.set_hexpand(true);
    let entry_count_label = Label::new(Some(""));
    entry_count_label.set_halign(Align::End);
    meta_row.append(&key_label);
    meta_row.append(&entry_count_label);
    write_vbox.append(&meta_row);

    let write_scroll = ScrolledWindow::builder()
        .vexpand(true)
        .build();
    let write_buf = TextBuffer::new(None);
    let write_view = TextView::builder()
        .buffer(&write_buf)
        .wrap_mode(WrapMode::Word)
        .top_margin(8)
        .bottom_margin(8)
        .left_margin(8)
        .right_margin(8)
        .monospace(false)
        .build();
    write_scroll.set_child(Some(&write_view));
    write_vbox.append(&write_scroll);

    let bot_row = GBox::new(Orientation::Horizontal, 8);
    bot_row.set_margin_top(4);
    let btn_clear = Button::with_label("🗑  Clear");
    let btn_save = Button::with_label("🔒  Encrypt & Save Entry");
    btn_save.set_halign(Align::End);
    btn_save.set_hexpand(true);
    bot_row.append(&btn_clear);
    bot_row.append(&btn_save);
    write_vbox.append(&bot_row);

    notebook.append_page(&write_vbox, Some(&Label::new(Some("✏  Write"))));

    // ── Decrypt tab ───────────────────────────────────────────────────────────
    let dec_vbox = GBox::new(Orientation::Vertical, 8);
    dec_vbox.set_margin_start(12);
    dec_vbox.set_margin_end(12);
    dec_vbox.set_margin_top(8);
    dec_vbox.set_margin_bottom(8);

    dec_vbox.append(&Label::builder()
        .label("Paste encrypted PGP block below:")
        .halign(Align::Start)
        .build());

    let cipher_scroll = ScrolledWindow::builder().height_request(200).build();
    let cipher_buf = TextBuffer::new(None);
    let cipher_view = TextView::builder()
        .buffer(&cipher_buf)
        .wrap_mode(WrapMode::None)
        .monospace(true)
        .top_margin(6)
        .left_margin(6)
        .build();
    cipher_scroll.set_child(Some(&cipher_view));
    dec_vbox.append(&cipher_scroll);

    let dec_btn_row = GBox::new(Orientation::Horizontal, 8);
    let btn_decrypt = Button::with_label("🔓  Decrypt");
    let btn_clear_dec = Button::with_label("Clear All");
    let decrypt_status = Label::new(Some(""));
    decrypt_status.set_halign(Align::End);
    decrypt_status.set_hexpand(true);
    dec_btn_row.append(&btn_decrypt);
    dec_btn_row.append(&btn_clear_dec);
    dec_btn_row.append(&decrypt_status);
    dec_vbox.append(&dec_btn_row);

    dec_vbox.append(&Label::builder()
        .label("Decrypted text (never saved):")
        .halign(Align::Start)
        .build());

    let plain_scroll = ScrolledWindow::builder().vexpand(true).build();
    let plain_buf = TextBuffer::new(None);
    let plain_view = TextView::builder()
        .buffer(&plain_buf)
        .wrap_mode(WrapMode::Word)
        .editable(false)
        .top_margin(8)
        .left_margin(8)
        .build();
    plain_scroll.set_child(Some(&plain_view));
    dec_vbox.append(&plain_scroll);

    dec_vbox.append(&Label::builder()
        .label("Plaintext is held in memory only and wiped on clear or close.")
        .halign(Align::Start)
        .css_classes(["dim-label"])
        .build());

    notebook.append_page(&dec_vbox, Some(&Label::new(Some("🔓  Decrypt"))));

    // ── Journal tab ───────────────────────────────────────────────────────────
    let journal_vbox = GBox::new(Orientation::Vertical, 8);
    journal_vbox.set_margin_start(12);
    journal_vbox.set_margin_end(12);
    journal_vbox.set_margin_top(8);
    journal_vbox.set_margin_bottom(8);

    let journal_btn_row = GBox::new(Orientation::Horizontal, 8);
    let btn_decrypt_all = Button::with_label("🔓  Decrypt All Entries");
    let btn_clear_journal = Button::with_label("Clear");
    let journal_status = Label::new(Some(""));
    journal_status.set_halign(Align::End);
    journal_status.set_hexpand(true);
    journal_btn_row.append(&btn_decrypt_all);
    journal_btn_row.append(&btn_clear_journal);
    journal_btn_row.append(&journal_status);
    journal_vbox.append(&journal_btn_row);

    let journal_scroll = ScrolledWindow::builder().vexpand(true).build();
    let journal_buf = TextBuffer::new(None);
    let journal_view = TextView::builder()
        .buffer(&journal_buf)
        .wrap_mode(WrapMode::Word)
        .editable(false)
        .top_margin(12)
        .bottom_margin(12)
        .left_margin(12)
        .right_margin(12)
        .build();
    journal_scroll.set_child(Some(&journal_view));
    journal_vbox.append(&journal_scroll);

    journal_vbox.append(&Label::builder()
        .label("Decrypted journal is held in memory only and wiped on clear or close.")
        .halign(Align::Start)
        .css_classes(["dim-label"])
        .build());

    notebook.append_page(&journal_vbox, Some(&Label::new(Some("📖  Journal"))));

    root.append(&notebook);
    window.set_child(Some(&root));

    // ── Load last journal ─────────────────────────────────────────────────────
    {
        let cfg = config::load();
        if let (Some(path), Some(key_id), Some(key_uid)) =
            (cfg.last_journal, cfg.last_key_id, cfg.last_key_uid)
        {
            if std::path::Path::new(&path).exists() {
                let mut s = state.borrow_mut();
                s.path = Some(path.clone());
                s.key_id = Some(key_id.clone());
                s.key_uid = Some(key_uid.clone());
                status_label.set_label(&format!("📖  {}", basename(&path)));
                key_label.set_label(&format!("Key: {}  ({})", key_id, key_uid));
                refresh_count(&path, &entry_count_label);
            }
        }
    }

    // ── New Journal ───────────────────────────────────────────────────────────
    {
        let state = state.clone();
        let window = window.clone();
        let status_label = status_label.clone();
        let key_label = key_label.clone();
        let entry_count_label = entry_count_label.clone();

        btn_new.connect_clicked(move |_| {
            let keys = gpg::list_public_keys();
            if keys.is_empty() {
                show_error(&window, "No public keys found in your GPG keyring.\n\nGenerate one with:\n  gpg --full-generate-key");
                return;
            }

            // File chooser
            let fc = FileChooserDialog::builder()
                .title("Choose journal file location")
                .transient_for(&window)
                .modal(true)
                .action(FileChooserAction::Save)
                .build();
            fc.add_button("Cancel", ResponseType::Cancel);
            fc.add_button("Next", ResponseType::Accept);
            fc.set_current_name("journal.md");

            let keys_c = keys.clone();
            let state = state.clone();
            let window = window.clone();
            let status_label = status_label.clone();
            let key_label = key_label.clone();
            let entry_count_label = entry_count_label.clone();

            fc.connect_response(move |fc, resp| {
                if resp != ResponseType::Accept {
                    fc.close();
                    return;
                }
                let path = match fc.file().and_then(|f| f.path()) {
                    Some(p) => p,
                    None => { fc.close(); return; }
                };
                let path_str = path.to_string_lossy().to_string();
                let path_str = if path_str.ends_with(".md") {
                    path_str
                } else {
                    format!("{}.md", path_str)
                };
                fc.close();

                // Key picker dialog
                let chosen = pick_key_dialog(&window, &keys_c);
                if let Some(key) = chosen {
                    // Create file if needed
                    if !std::path::Path::new(&path_str).exists() {
                        let header = format!(
                            "# Journal\n\n_Encrypted with PGP key `{}` ({})_\n\n",
                            key.key_id, key.uid
                        );
                        let _ = fs::write(&path_str, header);
                    }
                    let cfg = config::Config {
                        last_journal: Some(path_str.clone()),
                        last_key_id: Some(key.key_id.clone()),
                        last_key_uid: Some(key.uid.clone()),
                    };
                    config::save(&cfg);
                    let mut s = state.borrow_mut();
                    s.path = Some(path_str.clone());
                    s.key_id = Some(key.key_id.clone());
                    s.key_uid = Some(key.uid.clone());
                    status_label.set_label(&format!("📖  {}", basename(&path_str)));
                    key_label.set_label(&format!("Key: {}  ({})", key.key_id, key.uid));
                    refresh_count(&path_str, &entry_count_label);
                }
            });
            fc.show();
        });
    }

    // ── Open Journal ──────────────────────────────────────────────────────────
    {
        let state = state.clone();
        let window = window.clone();
        let status_label = status_label.clone();
        let key_label = key_label.clone();
        let entry_count_label = entry_count_label.clone();

        btn_open.connect_clicked(move |_| {
            let fc = FileChooserDialog::builder()
                .title("Open Journal File")
                .transient_for(&window)
                .modal(true)
                .action(FileChooserAction::Open)
                .build();
            fc.add_button("Cancel", ResponseType::Cancel);
            fc.add_button("Open", ResponseType::Accept);

            let state = state.clone();
            let window = window.clone();
            let status_label = status_label.clone();
            let key_label = key_label.clone();
            let entry_count_label = entry_count_label.clone();

            fc.connect_response(move |fc, resp| {
                if resp != ResponseType::Accept {
                    fc.close();
                    return;
                }
                let path = match fc.file().and_then(|f| f.path()) {
                    Some(p) => p,
                    None => { fc.close(); return; }
                };
                let path_str = path.to_string_lossy().to_string();
                fc.close();

                // Try to detect key from file header
                let detected = detect_key_from_file(&path_str);
                let key = if let Some(k) = detected {
                    Some(k)
                } else {
                    let keys = gpg::list_public_keys();
                    if keys.is_empty() {
                        show_error(&window, "No public keys found in your GPG keyring.");
                        return;
                    }
                    pick_key_dialog(&window, &keys)
                };

                if let Some(k) = key {
                    let cfg = config::Config {
                        last_journal: Some(path_str.clone()),
                        last_key_id: Some(k.key_id.clone()),
                        last_key_uid: Some(k.uid.clone()),
                    };
                    config::save(&cfg);
                    let mut s = state.borrow_mut();
                    s.path = Some(path_str.clone());
                    s.key_id = Some(k.key_id.clone());
                    s.key_uid = Some(k.uid.clone());
                    status_label.set_label(&format!("📖  {}", basename(&path_str)));
                    key_label.set_label(&format!("Key: {}  ({})", k.key_id, k.uid));
                    refresh_count(&path_str, &entry_count_label);
                }
            });
            fc.show();
        });
    }

    // ── Clear write box ───────────────────────────────────────────────────────
    {
        let write_buf = write_buf.clone();
        btn_clear.connect_clicked(move |_| {
            write_buf.set_text("");
        });
    }

    // ── Encrypt & Save ────────────────────────────────────────────────────────
    {
        let state = state.clone();
        let write_buf = write_buf.clone();
        let window = window.clone();
        let status_label = status_label.clone();
        let entry_count_label = entry_count_label.clone();

        btn_save.connect_clicked(move |_| {
            let s = state.borrow();
            let (path, key_id) = match (&s.path, &s.key_id) {
                (Some(p), Some(k)) => (p.clone(), k.clone()),
                _ => {
                    show_error(&window, "Please create or open a journal first.");
                    return;
                }
            };
            drop(s);

            let (start, end) = (write_buf.start_iter(), write_buf.end_iter());
            let text = write_buf.text(&start, &end, false).to_string();
            let text = text.trim().to_string();
            if text.is_empty() {
                show_error(&window, "Nothing to save.");
                return;
            }

            // Encrypt synchronously (GPG is fast for small text)
            match gpg::encrypt(&text, &key_id) {
                Ok(ciphertext) => {
                    let timestamp = Local::now().format("%Y-%m-%dT%H:%M").to_string();
                    let entry = format!("\n# {}\n\n{}\n", timestamp, ciphertext);
                    match OpenOptions::new().append(true).open(&path) {
                        Ok(mut f) => {
                            let _ = f.write_all(entry.as_bytes());
                            write_buf.set_text("");
                            refresh_count(&path, &entry_count_label);
                            status_label.set_label("✅  Entry saved!");
                            let sl = status_label.clone();
                            let p = path.clone();
                            glib::timeout_add_seconds_local(3, move || {
                                sl.set_label(&format!("📖  {}", basename(&p)));
                                glib::ControlFlow::Break
                            });
                        }
                        Err(e) => show_error(&window, &format!("Could not write to file:\n{e}")),
                    }
                }
                Err(e) => show_error(&window, &format!("Encryption failed:\n{e}")),
            }
        });
    }

    // ── Decrypt ───────────────────────────────────────────────────────────────
    {
        let cipher_buf = cipher_buf.clone();
        let plain_buf = plain_buf.clone();
        let decrypt_status = decrypt_status.clone();
        let window = window.clone();

        btn_decrypt.connect_clicked(move |_| {
            let (start, end) = (cipher_buf.start_iter(), cipher_buf.end_iter());
            let cipher_text = cipher_buf.text(&start, &end, false).to_string();
            let cipher_text = cipher_text.trim().to_string();
            if cipher_text.is_empty() { return; }

            decrypt_status.set_label("Decrypting…");

            match gpg::decrypt(&cipher_text) {
                Ok(plaintext) => {
                    plain_buf.set_text(&plaintext);
                    // plaintext (Zeroizing<String>) is dropped and zeroed here
                    decrypt_status.set_label("✅  Decrypted");
                }
                Err(e) => {
                    decrypt_status.set_label("❌  Failed");
                    show_error(&window, &format!("Decryption failed:\n{e}"));
                }
            }
        });
    }

    // ── Decrypt All ───────────────────────────────────────────────────────────
    {
        let state = state.clone();
        let journal_buf = journal_buf.clone();
        let journal_status = journal_status.clone();
        let window = window.clone();

        btn_decrypt_all.connect_clicked(move |_| {
            let path = match state.borrow().path.clone() {
                Some(p) => p,
                None => {
                    show_error(&window, "Please open a journal first.");
                    return;
                }
            };

            let contents = match fs::read_to_string(&path) {
                Ok(c) => c,
                Err(e) => {
                    show_error(&window, &format!("Could not read journal:\n{e}"));
                    return;
                }
            };

            journal_status.set_label("Decrypting…");
            journal_buf.set_text("");

            // Parse entries: split on "# YYYY-MM-DDT" headings
            let mut output = Zeroizing::new(String::new());
            let mut current_heading = String::new();
            let mut current_cipher = String::new();
            let mut in_pgp = false;
            let mut total = 0usize;
            let mut failed = 0usize;

            for line in contents.lines() {
                if line.starts_with("# 20") {
                    // flush previous entry
                    if !current_cipher.is_empty() {
                        total += 1;
                        match gpg::decrypt(&current_cipher) {
                            Ok(plain) => {
                                output.push_str(&current_heading);
                                output.push('\n');
                                output.push_str(plain.as_str());
                                output.push_str("\n---\n\n");
                            }
                            Err(_) => { failed += 1; }
                        }
                        current_cipher.clear();
                        in_pgp = false;
                    }
                    current_heading = line.to_string();
                } else if line.contains("-----BEGIN PGP MESSAGE-----") {
                    in_pgp = true;
                    current_cipher.push_str(line);
                    current_cipher.push('\n');
                } else if in_pgp {
                    current_cipher.push_str(line);
                    current_cipher.push('\n');
                    if line.contains("-----END PGP MESSAGE-----") {
                        in_pgp = false;
                    }
                }
            }
            // flush last entry
            if !current_cipher.is_empty() {
                total += 1;
                match gpg::decrypt(&current_cipher) {
                    Ok(plain) => {
                        output.push_str(&current_heading);
                        output.push('\n');
                        output.push_str(plain.as_str());
                        output.push_str("\n---\n\n");
                    }
                    Err(_) => { failed += 1; }
                }
            }

            journal_buf.set_text(output.as_str());

            let msg = if failed == 0 {
                format!("✅  {} entries decrypted", total)
            } else {
                format!("⚠  {} decrypted, {} failed", total - failed, failed)
            };
            journal_status.set_label(&msg);
            // output (Zeroizing) is dropped and zeroed here
        });
    }

    // ── Clear journal view ────────────────────────────────────────────────────
    {
        let journal_buf = journal_buf.clone();
        let journal_status = journal_status.clone();
        btn_clear_journal.connect_clicked(move |_| {
            journal_buf.set_text("");
            journal_status.set_label("");
        });
    }
    {
        let cipher_buf = cipher_buf.clone();
        let plain_buf = plain_buf.clone();
        let decrypt_status = decrypt_status.clone();

        btn_clear_dec.connect_clicked(move |_| {
            plain_buf.set_text("");
            cipher_buf.set_text("");
            decrypt_status.set_label("");
        });
    }

    // ── Wipe plaintext on close ───────────────────────────────────────────────
    {
        let plain_buf = plain_buf.clone();
        let write_buf = write_buf.clone();
        let journal_buf = journal_buf.clone();
        window.connect_close_request(move |_| {
            plain_buf.set_text("");
            write_buf.set_text("");
            journal_buf.set_text("");
            glib::Propagation::Proceed
        });
    }

    window.show();
}

// ── Helpers ───────────────────────────────────────────────────────────────────

fn basename(path: &str) -> String {
    std::path::Path::new(path)
        .file_name()
        .map(|n| n.to_string_lossy().to_string())
        .unwrap_or_else(|| path.to_string())
}

fn refresh_count(path: &str, label: &Label) {
    if let Ok(contents) = fs::read_to_string(path) {
        let count = contents.lines()
            .filter(|l| l.starts_with("# 20"))
            .count();
        label.set_label(&format!("{} {}", count,
            if count == 1 { "entry" } else { "entries" }));
    }
}

fn show_error(parent: &ApplicationWindow, msg: &str) {
    let d = gtk::AlertDialog::builder()
        .message("PGP Journal")
        .detail(msg)
        .modal(true)
        .build();
    d.show(Some(parent));
}

fn detect_key_from_file(path: &str) -> Option<gpg::KeyInfo> {
    let contents = fs::read_to_string(path).ok()?;
    let header: String = contents.chars().take(512).collect();
    let re = regex_key_id(&header)?;
    let keys = gpg::list_public_keys();
    keys.into_iter().find(|k| k.key_id.ends_with(&re) || re.ends_with(&k.key_id))
}

fn regex_key_id(text: &str) -> Option<String> {
    // Find `key \`HEXID\`` pattern written by the journal header
    let marker = "key `";
    let start = text.find(marker)? + marker.len();
    let end = text[start..].find('`')? + start;
    Some(text[start..end].to_string())
}

/// Show a key-picker dialog and return the chosen KeyInfo.
fn pick_key_dialog(parent: &ApplicationWindow, keys: &[gpg::KeyInfo]) -> Option<gpg::KeyInfo> {
    let labels: Vec<String> = keys.iter().map(|k| k.to_string()).collect();
    let string_list = StringList::new(&labels.iter().map(|s| s.as_str()).collect::<Vec<_>>());

    let dialog = Dialog::builder()
        .title("Select PGP Key")
        .transient_for(parent)
        .modal(true)
        .build();
    dialog.add_button("Cancel", ResponseType::Cancel);
    dialog.add_button("Select", ResponseType::Accept);

    let content = dialog.content_area();
    content.set_spacing(12);
    content.set_margin_start(16);
    content.set_margin_end(16);
    content.set_margin_top(16);
    content.set_margin_bottom(8);
    content.append(&Label::builder()
        .label("Select the PGP key for this journal:")
        .halign(Align::Start)
        .build());

    let dropdown = DropDown::new(Some(string_list), gtk::Expression::NONE);
    dropdown.set_selected(0);
    content.append(&dropdown);

    let result = Rc::new(RefCell::new(None));
    let result_c = result.clone();
    let keys_c = keys.to_vec();

    dialog.connect_response(move |d, resp| {
        if resp == ResponseType::Accept {
            let idx = dropdown.selected() as usize;
            if idx < keys_c.len() {
                *result_c.borrow_mut() = Some(keys_c[idx].clone());
            }
        }
        d.close();
    });

    dialog.show();

    // Run a local main loop until the dialog closes
    let ctx = glib::MainContext::default();
    while dialog.is_visible() {
        ctx.iteration(true);
    }

    result.borrow().clone()
}