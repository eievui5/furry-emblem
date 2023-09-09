#![cfg_attr(not(debug_assertions), windows_subsystem = "windows")] // hide console window on Windows in release

use eframe::IconData;
use egui::*;
use fe_editor::EditorApp;

const APP_NAME: &str = "Furry Emblem Editor";
const ICON: &[u8] = include_bytes!("../icon.png");

fn main() -> Result<(), eframe::Error> {
	tracing_subscriber::fmt::init();
	let options = eframe::NativeOptions {
		initial_window_size: Some(vec2(640.0, 480.0)),
		icon_data: load_icon(),
		..Default::default()
	};
	eframe::run_native(
		APP_NAME,
		options,
		Box::new(|_cc| Box::from(EditorApp::new())),
	)
}

fn load_icon() -> Option<IconData> {
	let image = image::load_from_memory(ICON).ok()?.into_rgba8();
	let (width, height) = image.dimensions();
	let rgba = image.into_raw();
	Some(IconData {
		rgba,
		width,
		height,
	})
}
