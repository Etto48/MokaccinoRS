use std::{sync::{Arc, Mutex}, time::Duration};

use eframe::{NativeOptions, egui::{Vec2, CentralPanel, load::SizedTexture, TextureOptions, Image}, CreationContext, epaint::{TextureHandle, Color32, Stroke}, egui::Frame};

use crate::{load_image, config::defines};

pub fn run(is_still_loading: Arc<Mutex<bool>>)
{
    let loading_image_size = Vec2::new(1400.0/2.0,256.0/2.0);

    eframe::run_native("Mokaccino", 
    NativeOptions {
        always_on_top: false,
        maximized: false,
        decorated: false,
        initial_window_size: Some(loading_image_size),
        resizable: false,
        transparent: true,
        mouse_passthrough: false,
        centered: true,
        icon_data: Some(super::load_icon::load_icon()),
        ..Default::default()
    }, Box::new(move |cc| Box::new(LoadingScreen::new(cc,is_still_loading,loading_image_size)))).unwrap();

}

pub struct LoadingScreen
{
    loading_image: TextureHandle,
    loading_image_size: Vec2,
    is_still_loading: Arc<Mutex<bool>>,
}

impl LoadingScreen
{
    pub fn new(cc: &CreationContext, is_still_loading: Arc<Mutex<bool>>, loading_image_size: Vec2) -> Self
    {
        let loading_image = cc.egui_ctx.load_texture("Loading", 
            load_image!("../../assets/loading.png"),
            TextureOptions::default());
        Self {
            loading_image,
            loading_image_size,
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
            let loading_image = SizedTexture::new(&self.loading_image, self.loading_image_size);
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
                ui.add(Image::new(loading_image));
            });
        }
        ctx.request_repaint_after(Duration::from_millis(defines::UPDATE_UI_INTERVAL_MS));
    }
}