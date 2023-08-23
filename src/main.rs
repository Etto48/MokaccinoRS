#![cfg_attr(not(debug_assertions), windows_subsystem = "windows")] // hide console window on Windows in release
use mokaccino::UI;
use eframe::{NativeOptions, epaint::Vec2};


fn main() {
    let options = NativeOptions{
        initial_window_size: Some(Vec2::new(800.0, 500.0)),
        min_window_size: Some(Vec2::new(400.0, 300.0)),
        ..Default::default()
    };
    if let Err(e) = eframe::run_native(
        "Mokaccino",
        options, 
        Box::new(|_cc| Box::<UI>::default()))
    {
        eprintln!("Error starting GUI: {}", e);
    }
}
