pub fn load_icon() -> eframe::IconData {
	let (icon_rgba, icon_width, icon_height) = {
		let icon = include_bytes!("../../assets/icon.png");
		let image = image::load_from_memory(icon)
			.expect("Failed to decode icon")
			.into_rgba8();
		let (width, height) = image.dimensions();
		let rgba = image.into_raw();
		(rgba, width, height)
	};
	
	eframe::IconData {
		rgba: icon_rgba,
		width: icon_width,
		height: icon_height,
	}
}