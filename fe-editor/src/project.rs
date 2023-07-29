use egui::*;
use serde::{Deserialize, Serialize};
use std::fs;
use std::io;
use std::mem;
use std::path::{Path, PathBuf};
use thiserror::Error;

pub const PROJECT_FILE: &str = "fe-project.toml";

#[derive(Debug, Error)]
pub enum LoadProjectError {
	#[error("failed to load {PROJECT_FILE}: {0}")]
	Open(io::Error),
	#[error("failed to parse {PROJECT_FILE}: {0}")]
	Parse(toml::de::Error),
}

#[derive(Clone, Default, Debug, Deserialize, Eq, PartialEq, Serialize)]
#[serde(default)]
pub struct Project {
	pub name: String,
	#[serde(skip)]
	pub path: PathBuf,
}

impl Project {
	fn new() -> Self {
		Self {
			name: String::from("New Project"),
			path: PathBuf::from("./"),
		}
	}

	/// Opens the project file within a given directory, given it exists.
	fn open_dir(path: impl AsRef<Path>) -> Result<Self, LoadProjectError> {
		use LoadProjectError::*;
		let path = path.as_ref();
		let project_file = fs::read_to_string(path.join(PROJECT_FILE)).map_err(|msg| Open(msg))?;
		let mut project: Self = toml::from_str(&project_file).map_err(|msg| Parse(msg))?;
		project.path = path.to_path_buf();
		Ok(project)
	}
}

pub struct NewProjectWindow {
	pub visible: bool,
	pub project: Project,
	pub dialog: egui_file::FileDialog,
}

impl Default for NewProjectWindow {
	fn default() -> Self {
		Self {
			visible: false,
			project: Project::new(),
			dialog: egui_file::FileDialog::select_folder(None),
		}
	}
}

impl NewProjectWindow {
	pub fn show(&mut self, ctx: &Context) -> Option<Project> {
		let mut project = None;

		Window::new("Create New Project")
			.open(&mut self.visible)
			.show(ctx, |ui| {
				ui.label("Project Name:");
				ui.text_edit_singleline(&mut self.project.name);
				ui.label("Path:");
				if ui.button(self.project.path.to_string_lossy()).clicked() {
					self.dialog.open();
				}
				if self.dialog.show(ctx).selected() {
					if let Some(path) = self.dialog.path().map(|p| p.to_path_buf()) {
						self.project.path = path;
					}
				}
				if ui.button("Create").clicked() {
					let mut new_project = Project::new();
					mem::swap(&mut new_project, &mut self.project);
					project = Some(new_project);
				}
			});

		project
	}
}

pub struct LoadProjectWindow {
	pub visible: bool,
	pub path: PathBuf,
	pub dialog: egui_file::FileDialog,
}

impl Default for LoadProjectWindow {
	fn default() -> Self {
		Self {
			visible: false,
			path: PathBuf::new(),
			dialog: egui_file::FileDialog::select_folder(None),
		}
	}
}

impl LoadProjectWindow {
	pub fn show(&mut self, ctx: &Context) -> Result<Option<Project>, LoadProjectError> {
		let mut project = Ok(None);

		Window::new("Add Project")
			.open(&mut self.visible)
			.show(ctx, |ui| {
				ui.label("Path:");
				if ui.button(self.path.to_string_lossy()).clicked() {
					self.dialog.open();
				}
				if self.dialog.show(ctx).selected() {
					if let Some(path) = self.dialog.path().map(|p| p.to_path_buf()) {
						self.path = path;
					}
				}
				if ui.button("Open").clicked() {
					project = Project::open_dir(&self.path).map(|p| Some(p));
				}
			});

		project
	}
}
