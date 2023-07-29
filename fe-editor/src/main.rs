#![cfg_attr(not(debug_assertions), windows_subsystem = "windows")] // hide console window on Windows in release

use egui::*;
use egui_toast::*;
use std::fs;
use std::mem;
use std::path::PathBuf;

mod editors;
use editors::*;

mod new_file_window;
use new_file_window::*;

mod project;
use project::*;

mod menu_bar;

const APP_NAME: &str = "Furry Emblem Editor";
const TOAST_LENGTH: f64 = 5.0;

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

fn error(text: WidgetText) -> Toast {
	Toast {
		text,
		kind: ToastKind::Error,
		options: ToastOptions::default()
			.duration_in_seconds(TOAST_LENGTH)
			.show_progress(true)
			.show_icon(true),
	}
}

#[derive(Default)]
pub struct EditorApp {
	// Options
	pub light_mode: bool,
	// File loading
	pub open_file_dialog: Option<egui_file::FileDialog>,
	pub new_file_window: NewFileWindow,
	pub opened_file: Option<PathBuf>,
	// Editor management
	pub primary_editor: Option<Box<dyn Editor>>,
	pub editors: Vec<Box<dyn Editor>>,
	// Project management
	pub local_projects: Vec<Project>,
	pub primary_project: Option<Project>,
	pub new_project_window: NewProjectWindow,
	pub load_project_window: LoadProjectWindow,
}

impl EditorApp {
	pub fn new() -> Self {
		let mut local_projects = Vec::new();

		if let Ok(dir) = fs::read_dir(".") {
			for entry in dir {
				let Ok(entry) = entry else {
					continue;
				};

				let fe_project = entry.path().join(PROJECT_FILE);

				let Ok(toml) = fs::read_to_string(fe_project) else {
					continue;
				};

				let Ok(mut project) = toml::from_str::<Project>(&toml) else {
					continue;
				};

				project.path = entry.path();

				local_projects.push(project);
			}
		}

		Self {
			local_projects,
			..Default::default()
		}
	}
}

impl eframe::App for EditorApp {
	fn update(&mut self, ctx: &Context, _frame: &mut eframe::Frame) {
		let mut toasts = Toasts::new()
			.anchor(Align2::CENTER_TOP, (0.0, 10.0))
			.direction(Direction::TopDown);

		menu_bar::show(self, &mut toasts, ctx);

		// New file window
		if let Some(new_editor) = self.new_file_window.show(ctx) {
			self.editors.push(new_editor);
		}

		SidePanel::left("Project Tree").show(ctx, |ui| {
			if let Some(project) = &self.primary_project {
				ui.heading(&project.name);
			} else {
				ui.label("No project loaded");
			}
			ui.separator();
			for project in &self.local_projects {
				if ui
					.button(format!("{} ({})", project.name, project.path.display()))
					.clicked()
				{}
			}
			ui.separator();
			if ui.button("Create new project").clicked() {
				self.new_project_window.visible = true;
			}
			if ui.button("Add existing project").clicked() {
				self.load_project_window.visible = true;
			}
		});

		if let Some(new_project) = self.new_project_window.show(ctx) {
			self.new_project_window.visible = false;
			self.primary_project = Some(new_project);
		}

		match self.load_project_window.show(ctx) {
			Ok(Some(project)) => {
				self.load_project_window.visible = false;
				self.primary_project = Some(project);
			}
			Err(msg) => {
				toasts.add(error(msg.to_string().into()));
			}
			_ => {}
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
						toasts.add(error(
							format!(
								"Failed to save {}: {msg}",
								self.editors[i].get_path().display()
							)
							.into(),
						));
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

fn editor_window_opts(
	toasts: &mut Toasts,
	ui: &mut Ui,
	editor: &mut dyn editors::Editor,
) -> Option<Box<dyn Editor>> {
	if ui.button("Save").clicked() {
		if let Err(msg) = editor.save() {
			toasts.add(error(
				format!("Failed to save {}:\n{msg}", editor.get_path().display()).into(),
			));
		}
	}

	if editor.is_toml() && ui.button("Open as TOML").clicked() {
		match TomlEditor::open(editor.get_path()) {
			Ok(toml_editor) => {
				return Some(Box::new(toml_editor));
			}
			Err(msg) => {
				toasts.add(error(
					format!(
						"Failed to reopen {} as TOML:\n{msg}\nHas the file moved?",
						editor.get_path().display()
					)
					.into(),
				));
			}
		}
	}

	None
}
