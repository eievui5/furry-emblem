use super::*;
use crate::file_dialogue::FilePicker;
use crate::impl_save_as;
use egui::*;
use fe_data::Map;
use std::fs;
use std::path::{Path, PathBuf};
use uuid::Uuid;

#[derive(Debug, Clone, Default)]
pub struct MapEditor {
	pub path: PathBuf,
	pub map: Map,
	pub source_map: Option<Map>,
	pub id: Uuid,
	pub icon_picker: FilePicker,
	pub icon: OptionalImage,
}

impl MapEditor {
	pub fn create(path: &Path) -> Box<dyn Editor> {
		Box::new(Self {
			path: path.to_path_buf(),
			id: Uuid::new_v4(),
			..Default::default()
		})
	}

	pub fn new(path: impl AsRef<Path>, text: &str) -> anyhow::Result<Self> {
		let map: Map = toml::from_str(text)?;
		let source_map = Some(map.clone());
		Ok(Self {
			path: path.as_ref().to_path_buf(),
			map,
			source_map,
			id: Uuid::new_v4(),
			..Default::default()
		})
	}
}

impl Editor for MapEditor {
	fn get_path(&self) -> &Path {
		&self.path
	}

	fn get_id(&self) -> Uuid {
		self.id
	}

	fn has_changes(&self) -> bool {
		self.source_map.as_ref().map_or(true, |s| self.map != *s)
	}

	impl_save_as!(map);

	fn show(&mut self, ui: &mut egui::Ui) {
		let scale: f32 = 32.0;

		let (_, rect) = ui.allocate_space(ui.available_size());
		let b = rect.min;
		let painter = ui.painter();

		for y in 0..10 {
			for x in 0..15 {
				let here = b + Vec2 {
					x: x as f32,
					y: y as f32,
				} * scale;
				painter.rect_filled(
					Rect {
						min: here,
						max: here + Vec2 { x: scale, y: scale },
					},
					0.0,
					if (x + y) % 2 == 0 {
						Color32::RED
					} else {
						Color32::BLUE
					},
				);
			}
		}
	}
}
