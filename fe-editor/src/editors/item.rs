use super::*;
use crate::file_dialogue::FilePicker;
use crate::impl_save_as;
use paste::paste;
use std::fs;
use std::num::NonZeroU32;
use std::path::{Path, PathBuf};
use uuid::Uuid;

#[derive(Clone, Debug, Default)]
pub struct ItemEditor {
	pub path: PathBuf,
	pub item: fe_data::Item,
	pub source_item: Option<fe_data::Item>,
	pub id: Uuid,
	pub icon_picker: FilePicker,
}

impl ItemEditor {
	pub fn create(path: &Path) -> Box<dyn Editor> {
		Box::new(Self {
			path: path.to_path_buf(),
			id: Uuid::new_v4(),
			..Default::default()
		})
	}

	pub fn new(path: impl AsRef<Path>, text: &str) -> anyhow::Result<Self> {
		let item: Item = toml::from_str(text)?;
		let source_item = Some(item.clone());
		Ok(Self {
			path: path.as_ref().to_path_buf(),
			item,
			source_item,
			id: Uuid::new_v4(),
			..Default::default()
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

				ui.label("Icon:");
				if ui.button(self.item.icon.path.to_string_lossy()).clicked() {
					self.icon_picker.open();
				}
				if let Some(path) = self.icon_picker.try_take_relative(&self.path) {
					self.item.icon.path = path
				}
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
								let func = $func;
								ui.close_menu();
								self.item.ty = func();
							}
						};
					}
					show_type!("None", || ItemType::None);
					show_type!("Weapon", || ItemType::Weapon(WeaponItem::default()));
				});
				ui.end_row();

				match &mut self.item.ty {
					ItemType::Heal(item) => {
						ui.label("Amount:");
						ui.add(egui::DragValue::new(&mut item.amount).speed(1));
						ui.end_row();
						ui.label("Uses:");
						ui.add(egui::DragValue::new(&mut item.uses).speed(1));
						ui.end_row();
					}
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
					ItemType::None => {}
				}
			});
	}
}
