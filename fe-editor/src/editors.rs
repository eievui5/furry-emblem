use fe_data::*;
use paste::paste;
use std::borrow::Cow;
use std::num::NonZeroU32;
use std::path::{Path, PathBuf};
use std::str::FromStr;
use std::{fs, io};
use thiserror::Error;
use uuid::Uuid;

pub fn open_editor(file: &Path, text: &str) -> Result<Box<dyn Editor>, EditorError> {
	use EditorError::*;

	let file_name = file.to_string_lossy();

	macro_rules! try_these {
		($($type:ident,)+$(,)?) => {
			$(
				if file_name.contains(concat!(".", stringify!($type))) {
					paste! {
						match [<$type:camel Editor>]::new(file, text) {
							Ok(editor) => {
								return Ok(Box::new(editor));
							}
							Err(msg) => {
								return Err(Parse(msg));
							}
						}
					}
				} else
			)+
			{
				return Err(UnknownFormat);
			}
		};
	}

	try_these!(
		item, class, // This should always be last because it never fails.
		toml,
	);
}

#[derive(Debug, Error)]
pub enum EditorError {
	#[error("Failed to open: {0}")]
	Open(io::Error),
	#[error("Failed to parse: {0}")]
	Parse(anyhow::Error),
	#[error("File did not match any expected formats")]
	UnknownFormat,
}

pub trait Editor {
	// Required methods.
	fn get_path(&self) -> &Path;
	fn get_id(&self) -> Uuid;
	fn show(&mut self, ui: &mut egui::Ui);
	fn save_as(&mut self, path: &Path) -> anyhow::Result<()>;

	fn has_changes(&self) -> bool {
		false
	}

	fn save(&mut self) -> anyhow::Result<()> {
		let path = self.get_path().to_path_buf();
		self.save_as(&path)
	}

	/// Determines whether or not an editor can be reopened as toml.
	fn is_toml(&self) -> bool {
		true
	}

	fn get_name(&self) -> Cow<str> {
		self.get_path()
			.file_name()
			.map_or(Cow::from("Unnamed File"), |p| p.to_string_lossy())
	}
}

#[derive(Debug, Error)]
enum SaveAsError {
	#[error("Cannot save a file with no name! (Hold shift to close without saving)")]
	NoName,
}

macro_rules! impl_save_as {
	($type:ident) => {
		paste! {
			fn save_as<'a>(&'a mut self, mut path: &'a Path) -> anyhow::Result<()> {
				if path.is_dir() {
					if self.$type.name.is_empty() {
						Err(SaveAsError::NoName)?;
					}
					self.path = path.join(format!("{}.{}.toml", self.$type.name, stringify!($type)));
					path = &self.path;
				}
				let text = toml::to_string(&self.$type)?;
				fs::write(path, text)?;
				self.[<source_ $type>] = Some(self.$type.clone());
				Ok(())
			}
		}
	};
}

#[derive(Clone, Debug, Default)]
pub struct ClassEditor {
	pub path: PathBuf,
	pub class: Class,
	pub source_class: Option<Class>,
	pub id: Uuid,
}

impl ClassEditor {
	pub fn create(path: &Path) -> Box<dyn Editor> {
		Box::new(Self {
			path: path.to_path_buf(),
			class: Default::default(),
			source_class: None,
			id: Uuid::new_v4(),
		})
	}

	pub fn new(path: impl AsRef<Path>, text: &str) -> anyhow::Result<Self> {
		let class: Class = toml::from_str(&text)?;
		let source_class = Some(class.clone());
		Ok(Self {
			path: path.as_ref().to_path_buf(),
			class,
			source_class,
			id: Uuid::new_v4(),
		})
	}
}

fn stat_editor(name: &str, stats: &mut Stats, ui: &mut egui::Ui) {
	ui.label(name);
	egui::Grid::new(name).min_col_width(50.0).show(ui, |ui| {
		ui.label("Hp");
		ui.label("Power");
		ui.label("Defense");
		ui.label("Resistance");
		ui.label("Dexterity");
		ui.label("Movement");
		ui.label("Constitution");
		ui.label("Reflexes");
		ui.end_row();

		parse_edit(ui, &mut stats.hp);
		parse_edit(ui, &mut stats.power);
		parse_edit(ui, &mut stats.defense);
		parse_edit(ui, &mut stats.resistance);
		parse_edit(ui, &mut stats.dexterity);
		parse_edit(ui, &mut stats.movement);
		parse_edit(ui, &mut stats.constitution);
		parse_edit(ui, &mut stats.reflexes);
		ui.end_row();
	});
}

fn parse_edit<I: FromStr + ToString + From<u8> + PartialEq>(ui: &mut egui::Ui, value: &mut I) {
	let mut string = if *value == 0.into() {
		String::new()
	} else {
		value.to_string()
	};
	ui.text_edit_singleline(&mut string);
	if string.is_empty() {
		*value = 0.into();
	} else if let Ok(result) = string.parse() {
		*value = result;
	}
}

impl Editor for ClassEditor {
	fn get_path(&self) -> &Path {
		&self.path
	}

	fn get_id(&self) -> Uuid {
		self.id
	}

	fn has_changes(&self) -> bool {
		self.source_class
			.as_ref()
			.map_or(true, |s| self.class != *s)
	}

	impl_save_as!(class);

	fn show(&mut self, ui: &mut egui::Ui) {
		egui::Grid::new("Class Grid").striped(true).show(ui, |ui| {
			ui.label("Name:");
			ui.text_edit_singleline(&mut self.class.name);
			ui.end_row();

			ui.label("Description:");
			ui.text_edit_multiline(&mut self.class.description);
			ui.end_row();

			stat_editor("Bases:", &mut self.class.bases, ui);
			ui.end_row();

			stat_editor("Growths:", &mut self.class.growths, ui);
			ui.end_row();
		});

		egui::Grid::new("Skill Grid")
			.min_col_width(100.0)
			.striped(true)
			.show(ui, |ui| {
				ui.label("Movement Skills:");
				ui.end_row();

				ui.checkbox(&mut self.class.leap, "Leap");
				ui.checkbox(&mut self.class.shove, "Shove");
				ui.checkbox(&mut self.class.canter, "Canter");
				ui.checkbox(&mut self.class.pass, "Pass");
				ui.checkbox(&mut self.class.pull, "Pull");
				ui.end_row();

				ui.label("Abilities:");
				ui.end_row();

				ui.checkbox(&mut self.class.focus, "Focus");
				ui.checkbox(&mut self.class.crit_boost, "Crit Boost");
				ui.checkbox(&mut self.class.battle_canter, "Battle Canter");
				ui.checkbox(&mut self.class.ferry, "Ferry");
				ui.checkbox(&mut self.class.zone_of_control, "Zone of Control");
				ui.end_row();

				ui.label("Weaknesses:");
				ui.end_row();

				ui.checkbox(&mut self.class.agile, "Agile");
				ui.checkbox(&mut self.class.flying, "Flying");
				ui.checkbox(&mut self.class.armored, "Armored");
				ui.end_row();
			});
	}
}

#[derive(Clone, Debug, Default)]
pub struct ItemEditor {
	pub path: PathBuf,
	pub item: fe_data::Item,
	pub source_item: Option<fe_data::Item>,
	pub id: Uuid,
}

impl ItemEditor {
	pub fn create(path: &Path) -> Box<dyn Editor> {
		Box::new(Self {
			path: path.to_path_buf(),
			item: Default::default(),
			source_item: None,
			id: Uuid::new_v4(),
		})
	}

	pub fn new(path: impl AsRef<Path>, text: &str) -> anyhow::Result<Self> {
		let item: Item = toml::from_str(&text)?;
		let source_item = Some(item.clone());
		Ok(Self {
			path: path.as_ref().to_path_buf(),
			item,
			source_item,
			id: Uuid::new_v4(),
		})
	}
}

impl Editor for ItemEditor {
	fn get_path(&self) -> &Path {
		&self.path
	}

	fn get_id(&self) -> Uuid {
		self.id
	}

	fn has_changes(&self) -> bool {
		self.source_item.as_ref().map_or(true, |i| *i != self.item)
	}

	impl_save_as!(item);

	fn show(&mut self, ui: &mut egui::Ui) {
		egui::Grid::new(0)
			.min_col_width(100.0)
			.striped(true)
			.show(ui, |ui| {
				ui.label("Name:");
				ui.text_edit_singleline(&mut self.item.name);
				ui.end_row();

				ui.label("Description:");
				ui.text_edit_multiline(&mut self.item.description);
				ui.end_row();

				let mut is_sellable = self.item.value.is_some();

				ui.label("Sellable:");
				ui.checkbox(&mut is_sellable, "");
				ui.end_row();

				if is_sellable {
					let mut value = self.item.value.map_or(1, |v| v.get());

					ui.label("Value:");
					ui.add(egui::DragValue::new(&mut value).speed(1));
					ui.end_row();

					self.item.value =
						Some(NonZeroU32::new(value).unwrap_or(NonZeroU32::new(1).unwrap()));
				} else {
					self.item.value = None;
				}

				ui.label("Type:");
				ui.menu_button(self.item.ty.to_string(), |ui| {
					macro_rules! show_type {
						($type:literal, $func:expr) => {
							if ui.button($type).clicked() {
								ui.close_menu();
								self.item.ty = $func();
							}
						};
					}
					show_type!("None", || { ItemType::None });
					show_type!("Weapon", || { ItemType::Weapon(WeaponItem::default()) });
				});
				ui.end_row();

				match &mut self.item.ty {
					ItemType::Weapon(item) => {
						ui.label("Damage:");
						ui.add(egui::DragValue::new(&mut item.damage).speed(1));
						ui.end_row();
						ui.label("Weight:");
						ui.add(egui::DragValue::new(&mut item.weight).speed(1));
						ui.end_row();
						ui.label("Durability:");
						ui.add(egui::DragValue::new(&mut item.durability).speed(1));
						ui.end_row();
					}
					_ => {}
				}
			});
	}
}

pub struct TomlEditor {
	pub path: PathBuf,
	pub text: String,
	pub id: Uuid,
}

impl TomlEditor {
	pub fn open(path: impl AsRef<Path>) -> io::Result<Self> {
		let text = fs::read_to_string(&path)?;
		Ok(Self::new(path, &text).unwrap())
	}

	pub fn new(path: impl AsRef<Path>, text: &str) -> anyhow::Result<Self> {
		Ok(Self {
			path: path.as_ref().to_path_buf(),
			text: text.to_string(),
			id: Uuid::new_v4(),
		})
	}
}

impl Editor for TomlEditor {
	fn get_path(&self) -> &Path {
		&self.path
	}

	fn get_id(&self) -> Uuid {
		self.id
	}

	fn is_toml(&self) -> bool {
		false
	}

	fn show(&mut self, ui: &mut egui::Ui) {
		egui::ScrollArea::vertical().show(ui, |ui| {
			ui.add(
				egui::TextEdit::multiline(&mut self.text)
					.code_editor()
					.desired_width(f32::INFINITY),
			)
		});
	}

	fn save_as(&mut self, path: &Path) -> anyhow::Result<()> {
		fs::write(path, &self.text)?;
		Ok(())
	}
}
