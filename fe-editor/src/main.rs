#![cfg_attr(not(debug_assertions), windows_subsystem = "windows")] // hide console window on Windows in release

use egui::*;
use fe_editor::EditorApp;

const APP_NAME: &str = "Furry Emblem Editor";

fn main() -> Result<(), eframe::Error> {
	let options = eframe::NativeOptions {
		initial_window_size: Some(vec2(640.0, 480.0)),
		..Default::default()
	};
	eframe::run_native(
		APP_NAME,
		options,
		Box::new(|_cc| Box::from(EditorApp::new())),
	)
}
