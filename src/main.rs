mod config;
mod gpg;
mod ui;

use gtk::prelude::*;
use gtk::{glib, Application};

const APP_ID: &str = "com.pilcrow.app";

fn main() -> glib::ExitCode {
    let app = Application::builder()
        .application_id(APP_ID)
        .build();

    app.connect_activate(ui::build_ui);
    app.run()
}