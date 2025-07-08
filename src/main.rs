mod encryption;
mod habit;
mod storage;
mod ui;
mod calendar;

use gtk4::prelude::*;
use gtk4::Application;
use ui::HabitApp;

const APP_ID: &str = "com.example.rust-gtk-habits";

fn main() {
    libadwaita::init().unwrap();
    
    let app = Application::builder()
        .application_id(APP_ID)
        .build();

    app.connect_activate(|app| {
        match HabitApp::new(app) {
            Ok(habit_app) => habit_app.show(),
            Err(e) => eprintln!("Failed to create app: {}", e),
        }
    });

    app.run();
}
