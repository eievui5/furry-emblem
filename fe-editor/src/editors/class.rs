use super::*;
use crate::impl_save_as;
use egui_extras::RetainedImage;
use std::fs;
use std::path::{Path, PathBuf};
use std::str::FromStr;
use uuid::Uuid;

#[derive(Debug, Clone, Default)]
pub struct ClassEditor {
	pub path: PathBuf,
	pub class: Class,
	pub source_class: Option<Class>,
	pub id: Uuid,
	pub icon_picker: FilePicker,
	pub icon: OptionalImage,
}

impl ClassEditor {
	pub fn create(path: &Path) -> Box<dyn Editor> {
		Box::new(Self {
			path: path.to_path_buf(),
			id: Uuid::new_v4(),
			..Default::default()
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
			..Default::default()
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

			ui.label("Icon:");
			ui.vertical(|ui| {
				if ui.button(self.class.icon.path.to_string_lossy()).clicked() {
					self.icon_picker.open();
				}
				if let Some(path) = self.icon_picker.try_take_relative(&self.path) {
					self.class.icon.path = path
				}
				if let Some(image) = &self.icon.0 {
					image.show(ui);
				} else if !self.class.icon.path.as_os_str().is_empty() {
					if let Ok(bytes) = fs::read(
						self.path
							.parent()
							.unwrap_or(Path::new(""))
							.join(&self.class.icon.path),
					) {
						self.icon.0 = Some(
							RetainedImage::from_image_bytes(
								self.class.icon.path.to_string_lossy(),
								&bytes,
							)
							.unwrap(),
						);
					}
				}
			});
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
