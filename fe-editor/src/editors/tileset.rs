use super::*;
use crate::file_dialogue::FilePicker;
use crate::impl_save_as;
use std::fs;
use std::path::{Path, PathBuf};
use uuid::Uuid;

#[derive(Debug, Clone, Default)]
pub struct TilesetEditor {
	pub path: PathBuf,
	pub tileset: Tileset,
	pub source_tileset: Option<Tileset>,
	pub id: Uuid,
	pub atlas_picker: FilePicker,
	pub atlas: OptionalImage,
}

impl TilesetEditor {
	pub fn create(path: &Path) -> Box<dyn Editor> {
		Box::new(Self {
			path: path.to_path_buf(),
			id: Uuid::new_v4(),
			..Default::default()
		})
	}

	pub fn new(path: impl AsRef<Path>, text: &str) -> anyhow::Result<Self> {
		let tileset: Tileset = toml::from_str(text)?;
		let source_tileset = Some(tileset.clone());
		Ok(Self {
			path: path.as_ref().to_path_buf(),
			tileset,
			source_tileset,
			id: Uuid::new_v4(),
			..Default::default()
		})
	}
}

impl Editor for TilesetEditor {
	fn get_path(&self) -> &Path {
		&self.path
	}

	fn get_id(&self) -> Uuid {
		self.id
	}

	fn has_changes(&self) -> bool {
		self.source_tileset
			.as_ref()
			.map_or(true, |s| self.tileset != *s)
	}

	impl_save_as!(tileset);

	fn show(&mut self, ui: &mut egui::Ui) {
		egui::Grid::new("Tileset Grid")
			.striped(true)
			.show(ui, |ui| {
				ui.label("Name:");
				ui.text_edit_singleline(&mut self.tileset.name);
				ui.end_row();

				ui.label("Tile Width/Height:");
				ui.add(egui::DragValue::new(&mut self.tileset.tile_width));
				ui.end_row();

				ui.label("Icon:");
				self.atlas.show(
					ui,
					&self.path,
					&mut self.tileset.texture.path,
					&mut self.atlas_picker,
				);
				ui.end_row();
			});

		egui::Grid::new("Tileset Atlas")
			.min_col_width(64.0)
			.striped(true)
			.show(ui, |ui| {
				// Table headers.
				ui.label("ID");
				ui.label("Terrain");
				ui.label("X");
				ui.label("Y");
				ui.end_row();

				enum Action {
					Clear(usize),
					Remove(usize),
				}

				let mut action = None;

				for (id, info) in self.tileset.atlas.iter_mut().enumerate() {
					if let TileEntry::Tile(info) = info {
						ui.label(&format!("Tile #{id}",));
						// TODO: Make this act like an enum, loading from some terrain table.
						let mut terrain = info.terrain.clone().unwrap_or(String::new());
						ui.text_edit_singleline(&mut terrain);
						info.terrain = if terrain.is_empty() {
							None
						} else {
							Some(terrain)
						};

						ui.add(egui::DragValue::new(&mut info.x));
						ui.add(egui::DragValue::new(&mut info.y));
						if ui.button("Clear").clicked() {
							action = Some(Action::Clear(id));
						}
						ui.end_row();
					} else {
						ui.label(&format!("Gap #{id}"))
							.on_hover_text("Deleted tiles leave gaps avoid shifting the tileset.");
						if ui.button("Remove").clicked() {
							action = Some(Action::Remove(id));
						}
						ui.end_row();
					}
				}

				match action {
					Some(Action::Clear(id)) => {
						self.tileset.atlas[id] = TileEntry::Gap;
					}
					Some(Action::Remove(id)) => {
						self.tileset.atlas.remove(id);
					}
					_ => {}
				}

				if ui.button("Create").clicked() {
					let gap = self
						.tileset
						.atlas
						.iter_mut()
						.enumerate()
						.filter(|x| matches!(x.1, TileEntry::Gap))
						.nth(0);
					let width = self
						.atlas
						.0
						.as_ref()
						.map(|a| {
							a.size()[0]
								/ (if self.tileset.tile_width > 0 {
									self.tileset.tile_width as usize
								} else {
									1
								})
						})
						.unwrap_or(usize::MAX);
					if let Some((id, gap)) = gap {
						*gap = TileEntry::Tile(TileInfo {
							x: (id % width) as u32,
							y: (id / width) as u32,
							..Default::default()
						});
					} else {
						let id = self.tileset.atlas.len();

						self.tileset.atlas.push(TileEntry::Tile(TileInfo {
							x: (id % width) as u32,
							y: (id / width) as u32,
							..Default::default()
						}))
					}
				}
			});
	}
}
