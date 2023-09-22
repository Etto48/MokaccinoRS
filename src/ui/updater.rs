use std::sync::{Arc, RwLock};

use eframe::egui::Context;

pub fn run(
    c: Context,
    running: Arc<RwLock<bool>>
)
{
    while running.read().unwrap().clone() {
        c.request_repaint();
        std::thread::sleep(std::time::Duration::from_millis(60));
    }
}