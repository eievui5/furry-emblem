use super::*;
use crate::file_dialogue::FilePicker;
use crate::impl_save_as;
use std::fs;
use std::path::{Path, PathBuf};
use uuid::Uuid;

#[derive(Debug, Clone, Default)]
pub struct UnitEditor {
	pub path: PathBuf,
	pub unit: Unit,
	pub source_unit: Option<Unit>,
	pub id: Uuid,
	pub icon_picker: FilePicker,
	pub icon: OptionalImage,
}

impl UnitEditor {
	pub fn create(path: &Path) -> Box<dyn Editor> {
		Box::new(Self {
			path: path.to_path_buf(),
			id: Uuid::new_v4(),
			..Default::default()
		})
	}

	pub fn new(path: impl AsRef<Path>, text: &str) -> anyhow::Result<Self> {
		let unit: Unit = toml::from_str(&text)?;
		let source_unit = Some(unit.clone());
		Ok(Self {
			path: path.as_ref().to_path_buf(),
			unit,
			source_unit,
			id: Uuid::new_v4(),
			..Default::default()
		})
	}
}

impl Editor for UnitEditor {
	fn get_path(&self) -> &Path {
		&self.path
	}

	fn get_id(&self) -> Uuid {
		self.id
	}

	fn has_changes(&self) -> bool {
		self.source_unit.as_ref().map_or(true, |s| self.unit != *s)
	}

	impl_save_as!(unit);

	fn show(&mut self, ui: &mut egui::Ui) {
		egui::Grid::new("Unit Grid").striped(true).show(ui, |ui| {
			ui.label("Name:");
			ui.text_edit_singleline(&mut self.unit.name);
			ui.end_row();

			ui.label("Description:");
			edit_optional(
				egui::Ui::text_edit_multiline,
				ui,
				&mut self.unit.description,
			);
			ui.end_row();

			ui.label("Affiliation:");
			edit_optional(
				egui::Ui::text_edit_singleline,
				ui,
				&mut self.unit.affiliation,
			);
			ui.end_row();

			stat_editor("Bases:", &mut self.unit.bases, ui);
			ui.end_row();

			stat_editor("Growths:", &mut self.unit.growths, ui);
			ui.end_row();
		});
	}
}
