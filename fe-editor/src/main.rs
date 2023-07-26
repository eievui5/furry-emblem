#![cfg_attr(not(debug_assertions), windows_subsystem = "windows")] // hide console window on Windows in release

use paste::paste;
use std::borrow::Cow;
use std::path::Path;
use std::path::PathBuf;
use std::{fs, mem};

mod editors;
use editors::*;

const APP_NAME: &str = "Furry Emblem Editor";

fn main() -> Result<(), eframe::Error> {
	let options = eframe::NativeOptions {
		initial_window_size: Some(egui::vec2(640.0, 480.0)),
		..Default::default()
	};
	eframe::run_native(
		APP_NAME,
		options,
		Box::new(|_cc| Box::<EditorApp>::default()),
	)
}

// This is a terrible terrible macro for generating what is not quite an enum but probably should be.
macro_rules! file_types {
	($($type:ident),+) => {
		const FILE_TYPE_STRINGS: &[&str] = &[
			$(stringify!($type)),+
		];

		paste! {
			const FILE_TYPE_EDITORS: &[fn(&Path) -> Box<dyn Editor>] = &[
				$([<$type Editor>]::create),+
			];
		}
	};
}

file_types!(Class, Item);

#[derive(Default)]
struct NewFileWindow {
	open: bool,
	path: Option<PathBuf>,
	ty: usize,
	dialog: Option<egui_file::FileDialog>,
}

impl NewFileWindow {
	fn open(&mut self, ty: usize) {
		self.open = true;
		self.path = None;
		self.dialog = None;
		self.ty = ty;
	}

	fn show(&mut self, ctx: &egui::Context) -> Option<Box<dyn Editor>> {
		let mut editor = None;
		let mut close_requested = false;

		egui::Window::new("New File")
			.open(&mut self.open)
			.show(ctx, |ui| {
				egui::Grid::new(0).min_col_width(100.0).show(ui, |ui| {
					ui.label("Path:");
					if ui
						.button(
							self.path
								.as_ref()
								.map_or(Cow::from("Select"), |p| p.to_string_lossy()),
						)
						.clicked()
					{
						let mut dialog = egui_file::FileDialog::save_file(self.path.clone());
						dialog.open();
						self.dialog = Some(dialog);
					}
					ui.end_row();

					ui.label("Type:");
					ui.menu_button(FILE_TYPE_STRINGS[self.ty], |ui| {
						for (i, name) in FILE_TYPE_STRINGS.iter().enumerate() {
							if ui.button(*name).clicked() {
								self.ty = i;
								ui.close_menu();
							}
						}
					});
					ui.end_row();
				});
				ui.horizontal(|ui| {
					if let Some(path) = &self.path {
						if ui.button("Create").clicked() {
							editor = Some(FILE_TYPE_EDITORS[self.ty](path));
							close_requested = true;
						}
					} else {
						ui.add_enabled(false, egui::Button::new("Create"));
					}
					if ui.button("Cancel").clicked() {
						close_requested = true;
					}
				})
			});

		self.open = self.open && !close_requested;

		if let Some(dialog) = &mut self.dialog {
			if dialog.show(ctx).selected() {
				self.path = dialog.path().map(|p| p.to_path_buf());
				self.dialog = None;
			}
		}

		editor
	}
}

struct EditorApp {
	open_file_dialog: Option<egui_file::FileDialog>,
	opened_file: Option<PathBuf>,
	primary_editor: Option<Box<dyn Editor>>,
	editors: Vec<Box<dyn Editor>>,
	log: String,
	light_mode: bool,
	new_file_window: NewFileWindow,
}

impl Default for EditorApp {
	fn default() -> Self {
		Self {
			open_file_dialog: None,
			opened_file: None,
			primary_editor: None,
			editors: Vec::new(),
			log: String::new(),
			light_mode: false,
			new_file_window: NewFileWindow::default(),
		}
	}
}

impl eframe::App for EditorApp {
	fn update(&mut self, ctx: &egui::Context, _frame: &mut eframe::Frame) {
		egui::TopBottomPanel::top("Menu Bar").show(ctx, |ui| {
			ui.horizontal(|ui| {
				ui.menu_button("File", |ui| {
					ui.menu_button("New", |ui| {
						for (i, name) in FILE_TYPE_STRINGS.iter().enumerate() {
							if ui.button(*name).clicked() {
								self.new_file_window.open(i);
								ui.close_menu();
							}
						}
					});
					if ui.button("Open").clicked() {
						let mut dialog = egui_file::FileDialog::save_file(self.opened_file.clone());
						dialog.open();
						self.open_file_dialog = Some(dialog);
						ui.close_menu();
					}
				});

				ui.menu_button("Options", |ui| {
					if self.light_mode {
						if ui.button("Switch to Dark Mode").clicked() {
							self.light_mode = false;
							ctx.set_visuals(egui::Visuals::dark());
							ui.close_menu();
						}
					} else if ui.button("Switch to Light Mode").clicked() {
						self.light_mode = true;
						ctx.set_visuals(egui::Visuals::light());
						ui.close_menu();
					}
				});
			});

			if let Some(dialog) = &mut self.open_file_dialog {
				if dialog.show(ctx).selected() {
					if let Some(file) = dialog.path() {
						match fs::read_to_string(&file) {
							Ok(text) => {
								macro_rules! try_these {
									($($type:ident,)+$(,)?) => {
										$(
											if let Ok(editor) = $type::new(&file, &text) {
												self.editors.push(Box::new(editor));
											} else
										)+
										{}
									};
								}

								try_these!(
									ItemEditor,
									ClassEditor,
									// This should always be last because it never fails.
									TomlEditor,
								);

								self.opened_file = Some(file.to_path_buf());
							}
							Err(msg) => {
								self.log = format!("Failed to open {}:\n{msg}", file.display());
							}
						}
					}
					self.open_file_dialog = None;
					ui.close_menu();
				}
			}
		});

		if !self.log.is_empty() {
			egui::TopBottomPanel::top("Log Panel").show(ctx, |ui| {
				ui.label(&self.log);
				if ui.button("Clear").clicked() {
					self.log = String::new();
				}
			});
		}

		// New file window
		if let Some(new_editor) = self.new_file_window.show(ctx) {
			self.editors.push(new_editor);
		}

		// Editor Windows
		for i in (0..self.editors.len()).rev() {
			let mut is_open = true;
			let mut close_requested = false;
			let mut primary_requested = false;

			let star = if self.editors[i].has_changes() {
				" *"
			} else {
				""
			};

			egui::Window::new(format!("{}{}", self.editors[i].get_name(), star))
				.id(egui::Id::new(self.editors[i].get_id()))
				.open(&mut is_open)
				.show(ctx, |ui| {
					ui.horizontal(|ui| {
						if let Some(new_editor) =
							editor_window_opts(&mut self.log, ui, &mut *self.editors[i])
						{
							close_requested = true;
							self.editors.push(new_editor);
						}
						primary_requested = ui.button("Make Primary").clicked();
					});

					ui.separator();

					self.editors[i].show(ui);
				});

			if primary_requested {
				if let Some(primary_editor) = &mut self.primary_editor {
					mem::swap(primary_editor, &mut self.editors[i]);
				} else {
					self.primary_editor = Some(self.editors.remove(i));
				}
			} else if close_requested || !is_open {
				if self.editors[i].has_changes() {
					if let Err(msg) = self.editors[i].save() {
						self.log = format!(
							"Failed to save {}: {msg}",
							self.editors[i].get_path().display()
						);
					} else {
						self.editors.remove(i);
					}
				} else {
					self.editors.remove(i);
				}
			}
		}

		egui::CentralPanel::default().show(ctx, |ui| {
			let mut pop_out_requested = false;
			let mut close_requested = false;

			if let Some(editor) = &mut self.primary_editor {
				ui.heading(editor.get_name());
				ui.separator();
				ui.horizontal(|ui| {
					if let Some(new_editor) = editor_window_opts(&mut self.log, ui, &mut **editor) {
						close_requested = true;
						self.editors.push(new_editor);
						return;
					}
					pop_out_requested = ui.button("Pop Out").clicked();
					close_requested = ui.button("Close").clicked();
				});
				ui.separator();
				editor.show(ui);
			}
			if pop_out_requested {
				let this_editor = self.primary_editor.take().unwrap();
				self.editors.push(this_editor);
			} else if close_requested {
				self.primary_editor = None;
			}
		});
	}
}

fn editor_window_opts(
	log: &mut String,
	ui: &mut egui::Ui,
	editor: &mut dyn editors::Editor,
) -> Option<Box<dyn Editor>> {
	if ui.button("Save").clicked() {
		if let Err(msg) = editor.save() {
			*log = format!("Failed to save {}:\n{msg}", editor.get_path().display())
		}
	}

	if editor.is_toml() && ui.button("Open as TOML").clicked() {
		match TomlEditor::open(editor.get_path()) {
			Ok(toml_editor) => {
				return Some(Box::new(toml_editor));
			}
			Err(msg) => {
				*log = format!(
					"Failed to reopen {} as TOML:\n{msg}\nHas the file moved?",
					editor.get_path().display()
				);
			}
		}
	}

	None
}
