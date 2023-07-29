use fe_data::*;
use std::borrow::Cow;
use std::num::NonZeroU32;
use std::path::{Path, PathBuf};
use std::str::FromStr;
use std::{fs, io};
use uuid::Uuid;

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

	fn show(&mut self, ui: &mut egui::Ui) {
		egui::Grid::new("Class Grid").striped(true).show(ui, |ui| {
			ui.label("Name:");
			ui.text_edit_singleline(&mut self.class.name);
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

	fn save_as(&mut self, path: &Path) -> anyhow::Result<()> {
		let text = toml::to_string(&self.class)?;
		fs::write(path, text)?;
		self.source_class = Some(self.class.clone());
		Ok(())
	}

	fn has_changes(&self) -> bool {
		self.source_class
			.as_ref()
			.map_or(true, |s| self.class != *s)
	}
}

pub struct ItemEditor {
	pub path: PathBuf,
	pub item: fe_data::Item,
	pub id: Uuid,
}

impl ItemEditor {
	pub fn create(path: &Path) -> Box<dyn Editor> {
		Box::new(Self {
			path: path.to_path_buf(),
			item: Default::default(),
			id: Uuid::new_v4(),
		})
	}

	pub fn new(path: impl AsRef<Path>, text: &str) -> anyhow::Result<Self> {
		let item = toml::from_str(&text)?;
		Ok(Self {
			path: path.as_ref().to_path_buf(),
			item,
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

	fn show(&mut self, ui: &mut egui::Ui) {
		egui::Grid::new(0)
			.min_col_width(100.0)
			.striped(true)
			.show(ui, |ui| {
				ui.label("Name:");
				ui.text_edit_singleline(&mut self.item.name);
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

	fn save_as(&mut self, path: &Path) -> anyhow::Result<()> {
		let text = toml::to_string(&self.item)?;
		fs::write(path, text)?;
		Ok(())
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
