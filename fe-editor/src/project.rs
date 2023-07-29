use egui::*;
use serde::{Deserialize, Serialize};
use std::path::{Path, PathBuf};
use std::{fs, io, mem};
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

#[derive(Default)]
pub struct ProjectManager {
	pub local_projects: Vec<Project>,
	pub primary_project: Option<Project>,
	pub new_project_window: NewProjectWindow,
	pub load_project_window: LoadProjectWindow,
}

impl ProjectManager {
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

	pub fn show(&mut self, ctx: &Context) -> Result<(), LoadProjectError> {
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

		if let Some(project) = self.load_project_window.show(ctx)? {
			self.load_project_window.visible = false;
			self.primary_project = Some(project);
		}

		Ok(())
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
