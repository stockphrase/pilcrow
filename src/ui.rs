use crate::{config, gpg};
use chrono::Local;
use gtk::prelude::*;
use gtk::{
    glib, Application, ApplicationWindow, Box as GBox, Button,
    Label, Notebook, ScrolledWindow, TextView, SearchBar, SearchEntry,
    TextBuffer, DropDown, StringList, Orientation, Align, WrapMode,
    Image, FileDialog, FileFilter, Window,
};
use gtk::gio;
use std::cell::RefCell;
use std::fs::{self, OpenOptions};
use std::io::Write;
use std::rc::Rc;
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
        .title("Pilcrow")
        .default_width(860)
        .default_height(680)
        .icon_name("pilcrow")
        .build();

    // ── Root container ────────────────────────────────────────────────────────
    let root = GBox::new(Orientation::Vertical, 0);

    // ── Top bar ───────────────────────────────────────────────────────────────
    let topbar = GBox::new(Orientation::Horizontal, 8);
    topbar.set_margin_start(12);
    topbar.set_margin_end(12);
    topbar.set_margin_top(8);
    topbar.set_margin_bottom(8);

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

    let write_scroll = ScrolledWindow::builder().vexpand(true).build();
    let write_buf = TextBuffer::new(None);
    let write_view = TextView::builder()
        .buffer(&write_buf)
        .wrap_mode(WrapMode::Word)
        .top_margin(8).bottom_margin(8)
        .left_margin(8).right_margin(8)
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

    // Search bar (shown/hidden with Ctrl+F)
    let search_bar = SearchBar::new();
    let search_entry = SearchEntry::new();
    search_entry.set_hexpand(true);
    let search_box = GBox::new(Orientation::Horizontal, 8);
    let match_label = Label::new(Some(""));
    match_label.set_halign(Align::End);
    let btn_prev = Button::with_label("▲");
    let btn_next = Button::with_label("▼");
    search_box.append(&search_entry);
    search_box.append(&btn_prev);
    search_box.append(&btn_next);
    search_box.append(&match_label);
    search_bar.set_child(Some(&search_box));
    search_bar.set_show_close_button(true);
    journal_vbox.append(&search_bar);

    let journal_scroll = ScrolledWindow::builder().vexpand(true).build();
    let journal_buf = TextBuffer::new(None);
    let journal_view = TextView::builder()
        .buffer(&journal_buf)
        .wrap_mode(WrapMode::Word)
        .editable(false)
        .top_margin(12).bottom_margin(12)
        .left_margin(12).right_margin(12)
        .build();
    journal_scroll.set_child(Some(&journal_view));
    journal_vbox.append(&journal_scroll);

    journal_vbox.append(&Label::builder()
        .label("Decrypted journal is held in memory only and wiped on clear or close.")
        .halign(Align::Start)
        .css_classes(["dim-label"]).build());

    notebook.append_page(&journal_vbox, Some(&Label::new(Some("🔓  Decrypt Journal"))));

    // Wire Ctrl+F to toggle the search bar
    let key_ctrl = gtk::EventControllerKey::new();
    {
        let search_bar = search_bar.clone();
        let search_entry = search_entry.clone();
        key_ctrl.connect_key_pressed(move |_, key, _, mods| {
            if key == gtk::gdk::Key::f
                && mods.contains(gtk::gdk::ModifierType::CONTROL_MASK)
            {
                let visible = !search_bar.is_search_mode();
                search_bar.set_search_mode(visible);
                if visible { search_entry.grab_focus(); }
                return glib::Propagation::Stop;
            }
            glib::Propagation::Proceed
        });
    }
    journal_view.add_controller(key_ctrl);

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

            let filter = FileFilter::new();
            filter.add_pattern("*.md");
            filter.set_name(Some("Markdown files"));

            let fc = FileDialog::builder()
                .title("Choose journal file location")
                .modal(true)
                .default_filter(&filter)
                .build();

            let keys_c = keys.clone();
            let state = state.clone();
            let window = window.clone();
            let status_label = status_label.clone();
            let key_label = key_label.clone();
            let entry_count_label = entry_count_label.clone();
            let win_ref: Option<&Window> = Some(window.upcast_ref());
            let window2 = window.clone();

            fc.save(win_ref, gio::Cancellable::NONE, move |res| {
                let file = match res { Ok(f) => f, Err(_) => return };
                let mut path_str = file.path()
                    .map(|p| p.to_string_lossy().to_string())
                    .unwrap_or_default();
                if !path_str.ends_with(".md") { path_str.push_str(".md"); }

                let chosen = pick_key_dialog(&window2, &keys_c);
                if let Some(key) = chosen {
                    if !std::path::Path::new(&path_str).exists() {
                        let header = format!(
                            "# Pilcrow Journal\n\n_Encrypted with PGP key `{}` ({})_\n\n",
                            key.key_id, key.uid
                        );
                        let _ = fs::write(&path_str, header);
                    }
                    config::save(&config::Config {
                        last_journal: Some(path_str.clone()),
                        last_key_id: Some(key.key_id.clone()),
                        last_key_uid: Some(key.uid.clone()),
                    });
                    let mut s = state.borrow_mut();
                    s.path = Some(path_str.clone());
                    s.key_id = Some(key.key_id.clone());
                    s.key_uid = Some(key.uid.clone());
                    status_label.set_label(&format!("📖  {}", basename(&path_str)));
                    key_label.set_label(&format!("Key: {}  ({})", key.key_id, key.uid));
                    refresh_count(&path_str, &entry_count_label);
                }
            });
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
            let filter = FileFilter::new();
            filter.add_pattern("*.md");
            filter.set_name(Some("Markdown files"));

            let fc = FileDialog::builder()
                .title("Open Journal File")
                .modal(true)
                .default_filter(&filter)
                .build();

            let state = state.clone();
            let window = window.clone();
            let status_label = status_label.clone();
            let key_label = key_label.clone();
            let entry_count_label = entry_count_label.clone();
            let win_ref: Option<&Window> = Some(window.upcast_ref());
            let window2 = window.clone();

            fc.open(win_ref, gio::Cancellable::NONE, move |res| {
                let file = match res { Ok(f) => f, Err(_) => return };
                let path_str = file.path()
                    .map(|p| p.to_string_lossy().to_string())
                    .unwrap_or_default();

                let detected = detect_key_from_file(&path_str);
                let key = if let Some(k) = detected {
                    Some(k)
                } else {
                    let keys = gpg::list_public_keys();
                    if keys.is_empty() {
                        show_error(&window2, "No public keys found in your GPG keyring.");
                        return;
                    }
                    pick_key_dialog(&window2, &keys)
                };

                if let Some(k) = key {
                    config::save(&config::Config {
                        last_journal: Some(path_str.clone()),
                        last_key_id: Some(k.key_id.clone()),
                        last_key_uid: Some(k.uid.clone()),
                    });
                    let mut s = state.borrow_mut();
                    s.path = Some(path_str.clone());
                    s.key_id = Some(k.key_id.clone());
                    s.key_uid = Some(k.uid.clone());
                    status_label.set_label(&format!("📖  {}", basename(&path_str)));
                    key_label.set_label(&format!("Key: {}  ({})", k.key_id, k.uid));
                    refresh_count(&path_str, &entry_count_label);
                }
            });
        });
    }

    // ── Clear write box ───────────────────────────────────────────────────────
    {
        let write_buf = write_buf.clone();
        btn_clear.connect_clicked(move |_| { write_buf.set_text(""); });
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
                _ => { show_error(&window, "Please create or open a journal first."); return; }
            };
            drop(s);

            let (start, end) = (write_buf.start_iter(), write_buf.end_iter());
            let text = write_buf.text(&start, &end, false).trim().to_string();
            if text.is_empty() { show_error(&window, "Nothing to save."); return; }

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

    // ── Decrypt All ───────────────────────────────────────────────────────────
    {
        let state = state.clone();
        let journal_buf = journal_buf.clone();
        let journal_status = journal_status.clone();
        let window = window.clone();

        btn_decrypt_all.connect_clicked(move |_| {
            let path = match state.borrow().path.clone() {
                Some(p) => p,
                None => { show_error(&window, "Please open a journal first."); return; }
            };
            let contents = match fs::read_to_string(&path) {
                Ok(c) => c,
                Err(e) => { show_error(&window, &format!("Could not read journal:\n{e}")); return; }
            };

            journal_status.set_label("Decrypting…");
            journal_buf.set_text("");

            let mut output = Zeroizing::new(String::new());
            let mut current_heading = String::new();
            let mut current_cipher = String::new();
            let mut in_pgp = false;
            let mut total = 0usize;
            let mut failed = 0usize;

            let flush = |heading: &str, cipher: &str, output: &mut Zeroizing<String>, failed: &mut usize, total: &mut usize| {
                if cipher.is_empty() { return; }
                *total += 1;
                match gpg::decrypt(cipher) {
                    Ok(plain) => {
                        output.push_str(heading);
                        output.push('\n');
                        output.push_str(plain.as_str());
                        output.push_str("\n---\n\n");
                    }
                    Err(_) => { *failed += 1; }
                }
            };

            for line in contents.lines() {
                if line.starts_with("# 20") {
                    flush(&current_heading, &current_cipher, &mut output, &mut failed, &mut total);
                    current_cipher.clear();
                    in_pgp = false;
                    current_heading = line.to_string();
                } else if line.contains("-----BEGIN PGP MESSAGE-----") {
                    in_pgp = true;
                    current_cipher.push_str(line);
                    current_cipher.push('\n');
                } else if in_pgp {
                    current_cipher.push_str(line);
                    current_cipher.push('\n');
                    if line.contains("-----END PGP MESSAGE-----") { in_pgp = false; }
                }
            }
            flush(&current_heading, &current_cipher, &mut output, &mut failed, &mut total);

            journal_buf.set_text(output.as_str());
            journal_status.set_label(&if failed == 0 {
                format!("✅  {} entries decrypted", total)
            } else {
                format!("⚠  {} decrypted, {} failed", total - failed, failed)
            });
        });
    }

    // ── Search ────────────────────────────────────────────────────────────────
    // Shared match positions: list of (start_offset, end_offset)
    let matches: Rc<RefCell<Vec<(i32, i32)>>> = Rc::new(RefCell::new(Vec::new()));
    let current_match: Rc<RefCell<usize>> = Rc::new(RefCell::new(0));

    // Create a highlight tag
    let _highlight_tag = journal_buf.create_tag(
        Some("search-highlight"),
        &[("background", &"#f5c542"), ("foreground", &"#000000")],
    );
    let _current_tag = journal_buf.create_tag(
        Some("search-current"),
        &[("background", &"#e8650a"), ("foreground", &"#ffffff")],
    );

    let do_search = {
        let journal_buf = journal_buf.clone();
        let matches = matches.clone();
        let current_match = current_match.clone();
        let match_label = match_label.clone();
        let journal_view = journal_view.clone();

        move |query: &str| {
            // Clear existing highlights
            let start = journal_buf.start_iter();
            let end = journal_buf.end_iter();
            journal_buf.remove_tag_by_name("search-highlight", &start, &end);
            journal_buf.remove_tag_by_name("search-current", &start, &end);

            let mut found = Vec::new();
            if !query.is_empty() {
                let text = journal_buf.text(&start, &end, false).to_lowercase();
                let q = query.to_lowercase();
                let mut pos = 0usize;
                while let Some(idx) = text[pos..].find(&q) {
                    let abs = pos + idx;
                    // byte offset to char offset
                    let char_start = text[..abs].chars().count() as i32;
                    let char_end = char_start + q.chars().count() as i32;
                    found.push((char_start, char_end));
                    pos = abs + q.len();
                }
                // Apply highlight tags
                for &(s, e) in &found {
                    let si = journal_buf.iter_at_offset(s);
                    let ei = journal_buf.iter_at_offset(e);
                    journal_buf.apply_tag_by_name("search-highlight", &si, &ei);
                }
            }

            let total = found.len();
            *matches.borrow_mut() = found;
            *current_match.borrow_mut() = 0;

            if total > 0 {
                // Highlight first match as current
                let m = matches.borrow();
                let si = journal_buf.iter_at_offset(m[0].0);
                let ei = journal_buf.iter_at_offset(m[0].1);
                journal_buf.apply_tag_by_name("search-current", &si, &ei);
                journal_view.scroll_to_iter(&mut journal_buf.iter_at_offset(m[0].0), 0.1, true, 0.0, 0.3);
                match_label.set_label(&format!("1 / {}", total));
            } else if !query.is_empty() {
                match_label.set_label("no matches");
            } else {
                match_label.set_label("");
            }
        }
    };

    // Search entry changed
    {
        let do_search = do_search.clone();
        search_entry.connect_search_changed(move |e| {
            do_search(&e.text());
        });
    }

    // Navigate to a specific match index
    let navigate = {
        let journal_buf = journal_buf.clone();
        let matches = matches.clone();
        let current_match = current_match.clone();
        let match_label = match_label.clone();
        let journal_view = journal_view.clone();

        move |delta: i32| {
            let m = matches.borrow();
            if m.is_empty() { return; }
            // Clear current highlight
            let start = journal_buf.start_iter();
            let end = journal_buf.end_iter();
            journal_buf.remove_tag_by_name("search-current", &start, &end);

            let total = m.len() as i32;
            let cur = *current_match.borrow() as i32;
            let next = ((cur + delta).rem_euclid(total)) as usize;
            *current_match.borrow_mut() = next;

            let si = journal_buf.iter_at_offset(m[next].0);
            let ei = journal_buf.iter_at_offset(m[next].1);
            journal_buf.apply_tag_by_name("search-current", &si, &ei);
            journal_view.scroll_to_iter(&mut journal_buf.iter_at_offset(m[next].0), 0.1, true, 0.0, 0.3);
            match_label.set_label(&format!("{} / {}", next + 1, total));
        }
    };

    {
        let navigate = navigate.clone();
        btn_next.connect_clicked(move |_| navigate(1));
    }
    {
        let navigate = navigate.clone();
        btn_prev.connect_clicked(move |_| navigate(-1));
    }

    // Clear search highlights when journal is cleared
    {
        let journal_buf = journal_buf.clone();
        let journal_status = journal_status.clone();
        let search_entry = search_entry.clone();
        let search_bar = search_bar.clone();
        let match_label_c = match_label.clone();
        let do_search = do_search.clone();
        btn_clear_journal.connect_clicked(move |_| {
            journal_buf.set_text("");
            journal_status.set_label("");
            search_bar.set_search_mode(false);
            search_entry.set_text("");
            match_label_c.set_label("");
            do_search("");
        });
    }

    // ── Wipe all plaintext on close and clear GPG agent cache ─────────────────
    {
        let write_buf = write_buf.clone();
        let journal_buf = journal_buf.clone();
        window.connect_close_request(move |_| {
            write_buf.set_text("");
            journal_buf.set_text("");
            gpg::clear_agent_cache();
            glib::Propagation::Proceed
        });
    }

    window.present();
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
        let count = contents.lines().filter(|l| l.starts_with("# 20")).count();
        label.set_label(&format!("{} {}", count, if count == 1 { "entry" } else { "entries" }));
    }
}

fn show_error(parent: &ApplicationWindow, msg: &str) {
    let d = gtk::AlertDialog::builder()
        .message("Pilcrow")
        .detail(msg)
        .modal(true)
        .build();
    d.show(Some(parent));
}

fn detect_key_from_file(path: &str) -> Option<gpg::KeyInfo> {
    let contents = fs::read_to_string(path).ok()?;
    let header: String = contents.chars().take(512).collect();
    let key_id = regex_key_id(&header)?;
    gpg::list_public_keys().into_iter()
        .find(|k| k.key_id.ends_with(&key_id) || key_id.ends_with(&k.key_id))
}

fn regex_key_id(text: &str) -> Option<String> {
    let marker = "key `";
    let start = text.find(marker)? + marker.len();
    let end = text[start..].find('`')? + start;
    Some(text[start..end].to_string())
}

fn pick_key_dialog(parent: &ApplicationWindow, keys: &[gpg::KeyInfo]) -> Option<gpg::KeyInfo> {
    let labels: Vec<String> = keys.iter().map(|k| k.to_string()).collect();
    let string_list = StringList::new(&labels.iter().map(|s| s.as_str()).collect::<Vec<_>>());

    let content = GBox::new(Orientation::Vertical, 12);
    content.set_margin_start(16);
    content.set_margin_end(16);
    content.set_margin_top(16);
    content.set_margin_bottom(16);
    content.append(&Label::builder()
        .label("Select the PGP key for this journal:")
        .halign(Align::Start).build());

    let dropdown = DropDown::new(Some(string_list), gtk::Expression::NONE);
    dropdown.set_selected(0);
    content.append(&dropdown);

    let btn_row = GBox::new(Orientation::Horizontal, 8);
    btn_row.set_halign(Align::End);
    let btn_cancel = Button::with_label("Cancel");
    let btn_select = Button::with_label("Select");
    btn_row.append(&btn_cancel);
    btn_row.append(&btn_select);
    content.append(&btn_row);

    let dialog = gtk::Window::builder()
        .title("Select PGP Key")
        .transient_for(parent)
        .modal(true)
        .resizable(false)
        .child(&content)
        .build();

    let result: Rc<RefCell<Option<gpg::KeyInfo>>> = Rc::new(RefCell::new(None));
    let result_c = result.clone();
    let keys_c = keys.to_vec();
    let dialog_c = dialog.clone();

    btn_select.connect_clicked(move |_| {
        let idx = dropdown.selected() as usize;
        if idx < keys_c.len() {
            *result_c.borrow_mut() = Some(keys_c[idx].clone());
        }
        dialog_c.close();
    });

    let dialog_c2 = dialog.clone();
    btn_cancel.connect_clicked(move |_| { dialog_c2.close(); });

    dialog.present();

    let ctx = glib::MainContext::default();
    while dialog.is_visible() { ctx.iteration(true); }

    let r = result.borrow().clone();
    r
}