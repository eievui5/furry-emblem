use egui::*;
use fe_data::*;
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
	#[error("failed to read content: {0}")]
	OpenContent(anyhow::Error),
}

pub struct ClassPreview {
	content: Class,
	path: PathBuf,
}

pub struct ItemPreview {
	content: Item,
	path: PathBuf,
}

pub struct Project {
	info: ProjectInfo,
	classes: Vec<ClassPreview>,
	items: Vec<ItemPreview>,
}

impl Project {
	fn show(&self, ui: &mut Ui) -> Option<PathBuf> {
		let mut result = None;

		if !self.classes.is_empty() {
			ui.label("Classes");
			for i in &self.classes {
				if ui.button(&i.content.name).clicked() {
					result = Some(i.path.clone());
				}
			}
			ui.separator();
		}

		if !self.items.is_empty() {
			ui.label("Items");
			for i in &self.items {
				if ui.button(&i.content.name).clicked() {
					result = Some(i.path.clone());
				}
			}
			ui.separator();
		}

		result
	}
}

impl TryFrom<ProjectInfo> for Project {
	type Error = anyhow::Error;

	fn try_from(info: ProjectInfo) -> Result<Self, anyhow::Error> {
		macro_rules! load_dir {
			($path:ident, $type:ident, $preview:ident) => {
				if let Ok(dir) = fs::read_dir(info.path.join(stringify!($path))) {
					for entry in dir {
						let entry = entry?;
						let entry_path = entry.path();
						if !entry_path.is_dir() {
							$path.push($preview {
								content: toml::from_str::<$type>(&fs::read_to_string(
									&entry_path,
								)?)?,
								path: entry_path,
							});
						}
					}
				}
			};
		}
		let mut classes = Vec::new();
		load_dir!(classes, Class, ClassPreview);
		let mut items = Vec::new();
		load_dir!(items, Item, ItemPreview);

		Ok(Self {
			classes,
			items,
			info,
		})
	}
}

#[derive(Clone, Default, Debug, Deserialize, Eq, PartialEq, Serialize)]
#[serde(default)]
pub struct ProjectInfo {
	pub name: String,
	#[serde(skip)]
	pub path: PathBuf,
}

impl ProjectInfo {
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
	pub local_projects: Vec<ProjectInfo>,
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

				let Ok(mut project) = toml::from_str::<ProjectInfo>(&toml) else {
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

	pub fn show(&mut self, ctx: &Context) -> Result<Option<PathBuf>, LoadProjectError> {
		use LoadProjectError::*;

		let mut result = Ok(None);

		SidePanel::left("Project Tree").show(ctx, |ui| {
			if let Some(project) = &self.primary_project {
				ui.heading(&project.info.name);
				ui.separator();
				result = Ok(project.show(ui));
			} else {
				ui.label("No project loaded");
				ui.separator();
				for project in &self.local_projects {
					if ui
						.button(format!("{} ({})", project.name, project.path.display()))
						.clicked()
					{
						match project.clone().try_into() {
							Ok(project) => self.primary_project = Some(project),
							Err(msg) => result = Err(OpenContent(msg)),
						}
					}
				}
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
			match new_project.clone().try_into() {
				Ok(project) => self.primary_project = Some(project),
				Err(msg) => result = Err(OpenContent(msg)),
			}
		}

		if let Some(project) = self.load_project_window.show(ctx)? {
			self.load_project_window.visible = false;
			match project.clone().try_into() {
				Ok(project) => self.primary_project = Some(project),
				Err(msg) => result = Err(OpenContent(msg)),
			}
		}

		result
	}
}

pub struct NewProjectWindow {
	pub visible: bool,
	pub project: ProjectInfo,
	pub dialog: egui_file::FileDialog,
}

impl Default for NewProjectWindow {
	fn default() -> Self {
		Self {
			visible: false,
			project: ProjectInfo::new(),
			dialog: egui_file::FileDialog::select_folder(None),
		}
	}
}

impl NewProjectWindow {
	pub fn show(&mut self, ctx: &Context) -> Option<ProjectInfo> {
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
					let mut new_project = ProjectInfo::new();
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
	pub fn show(&mut self, ctx: &Context) -> Result<Option<ProjectInfo>, LoadProjectError> {
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
					project = ProjectInfo::open_dir(&self.path).map(|p| Some(p));
				}
			});

		project
	}
}
