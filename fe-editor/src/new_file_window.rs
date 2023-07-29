use crate::editors::*;
use egui::*;
use paste::paste;
use std::borrow::Cow;
use std::path::{Path, PathBuf};

// This is a terrible terrible macro for generating what is not quite an enum but probably should be.
macro_rules! file_types {
	($($type:ident),+) => {
		pub const FILE_TYPE_STRINGS: &[&str] = &[
			$(stringify!($type)),+
		];

		paste! {
			const FILE_TYPE_EDITORS: &[fn(&Path) -> Box<dyn Editor>] = &[
				$([<$type Editor>]::create),+
			];
		}
	};
}

file_types!(Class, Item);

#[derive(Default)]
pub struct NewFileWindow {
	pub open: bool,
	pub path: Option<PathBuf>,
	pub ty: usize,
	pub dialog: Option<egui_file::FileDialog>,
}

impl NewFileWindow {
	pub fn open(&mut self, ty: usize) {
		self.open = true;
		self.path = None;
		self.dialog = None;
		self.ty = ty;
	}

	pub fn show(&mut self, ctx: &Context) -> Option<Box<dyn Editor>> {
		let mut editor = None;
		let mut close_requested = false;

		Window::new("New File")
			.open(&mut self.open)
			.show(ctx, |ui| {
				Grid::new(0).min_col_width(100.0).show(ui, |ui| {
					ui.label("Path:");
					if ui
						.button(
							self.path
								.as_ref()
								.map_or(Cow::from("Select"), |p| p.to_string_lossy()),
						)
						.clicked()
					{
						let mut dialog = egui_file::FileDialog::save_file(self.path.clone());
						dialog.open();
						self.dialog = Some(dialog);
					}
					ui.end_row();

					ui.label("Type:");
					ui.menu_button(FILE_TYPE_STRINGS[self.ty], |ui| {
						for (i, name) in FILE_TYPE_STRINGS.iter().enumerate() {
							if ui.button(*name).clicked() {
								self.ty = i;
								ui.close_menu();
							}
						}
					});
					ui.end_row();
				});
				ui.horizontal(|ui| {
					if let Some(path) = &self.path {
						if ui.button("Create").clicked() {
							editor = Some(FILE_TYPE_EDITORS[self.ty](path));
							close_requested = true;
						}
					} else {
						ui.add_enabled(false, Button::new("Create"));
					}
					if ui.button("Cancel").clicked() {
						close_requested = true;
					}
				})
			});

		self.open = self.open && !close_requested;

		if let Some(dialog) = &mut self.dialog {
			if dialog.show(ctx).selected() {
				self.path = dialog.path().map(|p| p.to_path_buf());
				self.dialog = None;
			}
		}

		editor
	}
}
