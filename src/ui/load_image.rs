#[macro_export]
macro_rules! load_image {
    ($path:expr) => {{
        use eframe::egui::ColorImage;
        let image = image::load_from_memory(include_bytes!($path))
            .expect("Failed to decode image")
            .into_rgba8();
        ColorImage::from_rgba_unmultiplied([image.width() as usize, image.height() as usize], image.into_raw().as_slice())
    }};
}