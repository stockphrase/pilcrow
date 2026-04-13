mod config;
mod gpg;
mod ui;

use gtk::prelude::*;
use gtk::{glib, Application};

const APP_ID: &str = "com.pilcrow";

fn main() -> glib::ExitCode {
    // Register the icon name so the taskbar finds it
    gtk::init().expect("Failed to initialise GTK");
    let display = gtk::gdk::Display::default().expect("No display");
    let theme = gtk::IconTheme::for_display(&display);
    theme.add_search_path(
        &format!("{}/.local/share/icons/hicolor", std::env::var("HOME").unwrap_or_default())
    );
    theme.add_search_path("/usr/share/icons/hicolor");

    let app = Application::builder()
        .application_id(APP_ID)
        .build();

    // Set the app name shown in the taskbar
    glib::set_application_name("Pilcrow");
    glib::set_prgname(Some("pilcrow"));

    app.connect_activate(|a| {
        // Set default icon for all windows
        gtk::Window::set_default_icon_name("pilcrow");
        ui::build_ui(a);
    });

    app.run()
}