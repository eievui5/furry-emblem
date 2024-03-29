use crate::file_dialogue::FilePicker;
use egui_extras::RetainedImage;
use fe_data::*;
use paste::paste;
use std::borrow::Cow;
use std::fs;
use std::path::Path;
use std::path::PathBuf;
use std::str::FromStr;
use std::{fmt, io};
use thiserror::Error;
use uuid::Uuid;

mod class;
pub use class::ClassEditor;

mod item;
pub use item::ItemEditor;

mod map;
pub use map::MapEditor;

mod tileset;
pub use tileset::TilesetEditor;

mod unit;
pub use unit::UnitEditor;

#[macro_export]
macro_rules! impl_save_as {
	($type:ident) => {
		paste! {
			fn save_as<'a>(&'a mut self, mut path: &'a Path) -> anyhow::Result<()> {
				if !path.exists() || path.is_dir() {
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

fn edit_optional<T>(
	with: impl Fn(&mut egui::Ui, &mut String) -> T,
	ui: &mut egui::Ui,
	string: &mut Option<String>,
) {
	let mut optional_string = string.as_ref().map_or_else(Default::default, Clone::clone);
	with(ui, &mut optional_string);
	if optional_string.is_empty() {
		*string = None;
	} else {
		*string = Some(optional_string);
	}
}

/// RetainedImage doesn't implement very basic traits (annoying) so we hafta do it for them.
#[derive(Default)]
pub struct OptionalImage(Option<RetainedImage>);

impl fmt::Debug for OptionalImage {
	fn fmt(&self, _: &mut fmt::Formatter<'_>) -> fmt::Result {
		Ok(())
	}
}

impl Clone for OptionalImage {
	fn clone(&self) -> Self {
		Self(None)
	}
}

impl OptionalImage {
	pub fn show(
		&mut self,
		ui: &mut egui::Ui,
		base_path: &Path,
		path: &mut PathBuf,
		picker: &mut FilePicker,
	) {
		if ui.button(path.to_string_lossy()).clicked() {
			picker.open();
		}
		if let Some(p) = picker.try_take_relative(base_path) {
			*path = p;
		}
		if let Some(image) = &self.0 {
			image.show(ui);
		} else if !path.as_os_str().is_empty() {
			if let Ok(bytes) = fs::read(base_path.parent().unwrap_or(Path::new("")).join(&path)) {
				self.0 =
					Some(RetainedImage::from_image_bytes(path.to_string_lossy(), &bytes).unwrap());
			}
		}
	}
}

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

	try_these! {
		map,
		item,
		class,
		tileset,
		unit,
	};
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
