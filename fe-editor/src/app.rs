use crate::editors::{self, Editor, TomlEditor};
use crate::menu_bar::{self, FileTab, OptionsTab};
use crate::project::*;
use egui::*;
use egui_toast::*;
use std::mem;

const TOAST_LENGTH: f64 = 5.0;

fn error(text: impl ToString) -> Toast {
	Toast {
		text: text.to_string().into(),
		kind: ToastKind::Error,
		options: ToastOptions::default()
			.duration_in_seconds(TOAST_LENGTH)
			.show_progress(true)
			.show_icon(true),
	}
}

#[derive(Default)]
pub struct EditorApp {
	pub file: FileTab,
	pub options: OptionsTab,
	// Editor management
	pub primary_editor: Option<Box<dyn Editor>>,
	pub editors: Vec<Box<dyn Editor>>,
	// Project management
	pub project_manager: ProjectManager,
}

impl EditorApp {
	pub fn new() -> Self {
		Self::default()
	}
}

impl eframe::App for EditorApp {
	fn update(&mut self, ctx: &Context, _frame: &mut eframe::Frame) {
		// Keep this function as small as possible.
		// Anything substantial should be refactored elsewhere.
		// Make sure to move any member variables out of EditorApp too, if possible.
		// Ideally child components should not need access to EditorApp.

		// DO NOT pass `toasts` to children.
		// They should be responsible for passing errors up to this function.
		let mut toasts = Toasts::new()
			.anchor(Align2::CENTER_TOP, (0.0, 10.0))
			.direction(Direction::TopDown);

		if let Err(msg) = menu_bar::show(&mut self.file, &mut self.options, ctx) {
			toasts.add(error(msg));
		}

		// New file window
		if let Some(new_editor) = self.file.new_file_window.show(ctx) {
			self.editors.push(new_editor);
		}

		if let Err(msg) = self.project_manager.show(ctx) {
			toasts.add(error(msg));
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

			Window::new(format!("{}{}", self.editors[i].get_name(), star))
				.id(Id::new(self.editors[i].get_id()))
				.open(&mut is_open)
				.show(ctx, |ui| {
					ui.horizontal(|ui| {
						if let Some(new_editor) =
							editor_window_opts(&mut toasts, ui, &mut *self.editors[i])
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
						toasts.add(error(format!(
							"Failed to save {}: {msg}",
							self.editors[i].get_path().display()
						)));
					} else {
						self.editors.remove(i);
					}
				} else {
					self.editors.remove(i);
				}
			}
		}

		CentralPanel::default().show(ctx, |ui| {
			let mut pop_out_requested = false;
			let mut close_requested = false;

			if let Some(editor) = &mut self.primary_editor {
				ui.heading(editor.get_name());
				ui.separator();
				ui.horizontal(|ui| {
					if let Some(new_editor) = editor_window_opts(&mut toasts, ui, &mut **editor) {
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

		toasts.show(ctx);
	}
}

pub fn editor_window_opts(
	toasts: &mut Toasts,
	ui: &mut Ui,
	editor: &mut dyn editors::Editor,
) -> Option<Box<dyn Editor>> {
	if ui.button("Save").clicked() {
		if let Err(msg) = editor.save() {
			toasts.add(error(format!(
				"Failed to save {}:\n{msg}",
				editor.get_path().display()
			)));
		}
	}

	if editor.is_toml() && ui.button("Open as TOML").clicked() {
		match TomlEditor::open(editor.get_path()) {
			Ok(toml_editor) => {
				return Some(Box::new(toml_editor));
			}
			Err(msg) => {
				toasts.add(error(format!(
					"Failed to reopen {} as TOML:\n{msg}\nHas the file moved?",
					editor.get_path().display()
				)));
			}
		}
	}

	None
}
