use std::{sync::{Arc, Mutex}, time::Duration};

use eframe::{NativeOptions, egui::{Vec2, CentralPanel, load::SizedTexture, TextureOptions, Image}, CreationContext, epaint::{TextureHandle, Color32, Stroke}, egui::Frame};

use crate::{load_image, config::defines};

pub fn run(is_still_loading: Arc<Mutex<bool>>)
{
    eframe::run_native("Mokaccino", 
    NativeOptions {
        always_on_top: false,
        maximized: false,
        decorated: false,
        initial_window_size: Some(Vec2::new(256.0, 256.0)),
        resizable: false,
        transparent: true,
        mouse_passthrough: false,
        centered: true,
        icon_data: Some(super::load_icon::load_icon()),
        ..Default::default()
    }, Box::new(|cc| Box::new(LoadingScreen::new(cc,is_still_loading)))).unwrap();

}

pub struct LoadingScreen
{
    loading_image: TextureHandle,
    is_still_loading: Arc<Mutex<bool>>,
}

impl LoadingScreen
{
    pub fn new(cc: &CreationContext, is_still_loading: Arc<Mutex<bool>>) -> Self
    {
        let loading_image = cc.egui_ctx.load_texture("Loading", 
            load_image!("../../assets/icon.png"),
            TextureOptions::default());
        Self {
            loading_image,
            is_still_loading,
        }
    }
}

impl eframe::App for LoadingScreen
{
    fn clear_color(&self, _visuals: &eframe::egui::Visuals) -> [f32; 4] {
        [0.0, 0.0, 0.0, 0.0]
    }

    fn update(&mut self, ctx: &eframe::egui::Context, frame: &mut eframe::Frame) {
        if *self.is_still_loading.lock().unwrap() == false
        {
            frame.close();
        }
        else
        {
            CentralPanel::default()
            .frame(
                Frame
                {
                    fill: Color32::TRANSPARENT,
                    stroke: Stroke::NONE,
                    ..Default::default()
                }
            )
            .show(ctx, |ui|{
                let loading_image = SizedTexture::new(&self.loading_image, Vec2::new(256.0, 256.0));
                ui.add(Image::new(loading_image));
            });
        }
        ctx.request_repaint_after(Duration::from_millis(defines::UPDATE_UI_INTERVAL_MS));
    }
}