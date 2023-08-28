use crate::editors::*;
use egui::*;
use fe_data::Module;
use fe_data::*;
use notify::{RecommendedWatcher, RecursiveMode, Watcher};
use paste::paste;
use std::path::{Path, PathBuf};
use std::sync::mpsc::{channel, Receiver};
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

#[derive(Clone, Default, Eq, PartialEq)]
pub struct ClassPreview {
	content: Class,
	path: PathBuf,
}

#[derive(Clone, Default, Eq, PartialEq)]
pub struct ItemPreview {
	content: Item,
	path: PathBuf,
}

#[derive(Clone, Default, Eq, PartialEq)]
pub struct MapPreview {
	content: Map,
	path: PathBuf,
}

#[derive(Clone, Default, Eq, PartialEq)]
pub struct UnitPreview {
	content: Unit,
	path: PathBuf,
}

#[derive(Clone, Default, Eq, PartialEq)]
pub struct Project {
	info: Module,
	classes: Vec<ClassPreview>,
	items: Vec<ItemPreview>,
	maps: Vec<MapPreview>,
	units: Vec<UnitPreview>,
}

pub enum ProjectShowResponse {
	None,
	Open(PathBuf),
	Delete(PathBuf),
	New(Box<dyn Editor>),
}

impl Project {
	fn show(&self, ui: &mut Ui) -> ProjectShowResponse {
		use ProjectShowResponse::*;

		let mut result = None;

		macro_rules! show_type {
			($member:ident, $editor:ident) => {
				paste! {
					ui.collapsing(stringify!([<$member:camel>]), |ui| {
						for i in &self.$member {
							ui.horizontal(|ui| {
								if ui.link("\u{1F5D9}").clicked() {
									result = Delete(i.path.clone());
								}
								if ui.button(&i.content.name).clicked() {
									result = Open(i.path.clone());
								}
							});
						}
						ui.separator();
						if ui.button("Create New").clicked() {
							let mut editor = $editor::default();
							editor.path = self.info.path.join(stringify!($member)).to_path_buf();
							result = New(Box::new(editor));
						}
					});
				}
			};
		}

		show_type!(classes, ClassEditor);
		show_type!(items, ItemEditor);
		show_type!(maps, MapEditor);
		show_type!(units, UnitEditor);

		result
	}

	fn populate(&mut self) -> Result<(), anyhow::Error> {
		macro_rules! load_dir {
			($path:ident, $type:ident, $preview:ident) => {
				let path = stringify!($path);
				fs::create_dir_all(self.info.path.join(path))?;
				let dir = fs::read_dir(self.info.path.join(path))?;
				self.$path.clear();
				for entry in dir {
					let entry = entry?;
					let entry_path = entry.path();
					if !entry_path.is_dir() {
						self.$path.push($preview {
							content: toml::from_str::<$type>(&fs::read_to_string(&entry_path)?)?,
							path: entry_path,
						});
					}
				}
			};
		}
		load_dir!(classes, Class, ClassPreview);
		load_dir!(items, Item, ItemPreview);
		load_dir!(units, Unit, UnitPreview);
		Ok(())
	}
}

impl TryFrom<Module> for Project {
	type Error = anyhow::Error;

	fn try_from(info: Module) -> Result<Self, anyhow::Error> {
		let mut project = Self {
			classes: Vec::new(),
			items: Vec::new(),
			maps: Vec::new(),
			units: Vec::new(),
			info,
		};

		project.populate()?;

		Ok(project)
	}
}

fn make_new_module() -> Module {
	Module {
		name: String::from("New Project"),
		path: PathBuf::from("./"),
		..Default::default()
	}
}

/// Opens the project file within a given directory, given it exists.
fn open_module(path: impl AsRef<Path>) -> Result<Module, LoadProjectError> {
	use LoadProjectError::*;
	let path = path.as_ref();
	let project_file = fs::read_to_string(path.join(PROJECT_FILE)).map_err(Open)?;
	let mut project: Module = toml::from_str(&project_file).map_err(Parse)?;
	project.path = path.to_path_buf();
	Ok(project)
}

pub struct ProjectManager {
	pub local_projects: Vec<Module>,
	pub source_project: Option<Module>,
	pub primary_project: Option<Project>,
	pub new_project_window: NewProjectWindow,
	pub load_project_window: LoadProjectWindow,
	pub folder_watcher: Option<RecommendedWatcher>,
	pub needs_update: Option<Receiver<notify::Result<notify::Event>>>,
}

impl ProjectManager {
	pub fn set_project(&mut self, project: Project) {
		let (sender, needs_update) = channel();

		self.folder_watcher = notify::recommended_watcher(move |res| {
			if let Err(msg) = sender.send(res) {
				// There's no real way to handle this, but we can at least print an error message.
				// Ideally we'd show a toast but clearly cross-thread communication isn't working so that's not an option.
				eprintln!("Failed to notify main thread of folder update: {msg}");
			}
		})
		.ok();

		if let Some(folder_watcher) = &mut self.folder_watcher {
			if let Err(msg) = folder_watcher.watch(&project.info.path, RecursiveMode::Recursive) {
				eprintln!("Failed to watch {}: {msg}", project.info.path.display());
			}
		}

		self.source_project = Some(project.info.clone());
		self.primary_project = Some(project);

		self.needs_update = Some(needs_update);
	}

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

				let Ok(mut project) = toml::from_str::<Module>(&toml) else {
					continue;
				};

				project.path = entry.path();

				local_projects.push(project);
			}
		}

		Self {
			local_projects,
			source_project: None,
			primary_project: None,
			new_project_window: NewProjectWindow::default(),
			load_project_window: LoadProjectWindow::default(),
			folder_watcher: None,
			needs_update: None,
		}
	}

	pub fn show(&mut self, ctx: &Context) -> Result<ProjectShowResponse, LoadProjectError> {
		use LoadProjectError::*;

		let mut result = Ok(ProjectShowResponse::None);

		if let (Some(primary_project), Some(needs_update)) =
			(&mut self.primary_project, &self.needs_update)
		{
			if let Ok(Ok(notify::Event { kind, .. })) = needs_update.try_recv() {
				if let notify::EventKind::Create(..) | notify::EventKind::Remove(..) = kind {
					primary_project.populate().map_err(OpenContent)?;
				}
			}
		}

		SidePanel::left("Project Tree").show(ctx, |ui| {
			if let Some(project) = &mut self.primary_project {
				ui.text_edit_singleline(&mut project.info.name);
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
						match TryInto::<Project>::try_into(project.clone()) {
							Ok(project) => {
								self.set_project(project);
								break;
							}
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
				Ok(project) => self.set_project(project),
				Err(msg) => result = Err(OpenContent(msg)),
			}
		}

		if let Some(project) = self.load_project_window.show(ctx)? {
			self.load_project_window.visible = false;
			match project.clone().try_into() {
				Ok(project) => self.set_project(project),
				Err(msg) => result = Err(OpenContent(msg)),
			}
		}

		result
	}

	pub fn has_changes(&self) -> bool {
		self.source_project.as_ref() != self.primary_project.as_ref().map(|p| &p.info)
	}

	pub fn save(&mut self) -> anyhow::Result<()> {
		if let Some(primary_project) = &self.primary_project {
			let text = toml::to_string(&primary_project.info)?;
			fs::write(primary_project.info.path.join(PROJECT_FILE), text)?;
			self.source_project = Some(primary_project.info.clone());
		}
		Ok(())
	}
}

pub struct NewProjectWindow {
	pub visible: bool,
	pub project: Module,
	pub dialog: egui_file::FileDialog,
}

impl Default for NewProjectWindow {
	fn default() -> Self {
		Self {
			visible: false,
			project: make_new_module(),
			dialog: egui_file::FileDialog::select_folder(None),
		}
	}
}

impl NewProjectWindow {
	pub fn show(&mut self, ctx: &Context) -> Option<Module> {
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
					let mut new_project = make_new_module();
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
	pub fn show(&mut self, ctx: &Context) -> Result<Option<Module>, LoadProjectError> {
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
					project = open_module(&self.path).map(Some);
				}
			});

		project
	}
}
