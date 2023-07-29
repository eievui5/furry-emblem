use crate::editors::*;
use crate::new_file_window::{NewFileWindow, FILE_TYPE_STRINGS};
use egui::*;
use paste::paste;
use std::fs;
use std::path::PathBuf;

#[derive(Default)]
pub struct FileTab {
	pub open_file_dialog: Option<egui_file::FileDialog>,
	pub new_file_window: NewFileWindow,
	pub opened_file: Option<PathBuf>,
}

#[derive(Default)]
pub struct OptionsTab {
	pub light_mode: bool,
}

type ShowResult = Result<Option<Box<dyn Editor>>, EditorError>;

pub fn show(file_tab: &mut FileTab, options: &mut OptionsTab, ctx: &egui::Context) -> ShowResult {
	use EditorError::*;

	let mut result: ShowResult = Ok(None);

	TopBottomPanel::top("Menu Bar").show(ctx, |ui| {
		ui.horizontal(|ui| {
			ui.menu_button("File", |ui| {
				ui.menu_button("New", |ui| {
					for (i, name) in FILE_TYPE_STRINGS.iter().enumerate() {
						if ui.button(*name).clicked() {
							file_tab.new_file_window.open(i);
							ui.close_menu();
						}
					}
				});
				if ui.button("Open").clicked() {
					let mut dialog = egui_file::FileDialog::save_file(file_tab.opened_file.clone());
					dialog.open();
					file_tab.open_file_dialog = Some(dialog);
					ui.close_menu();
				}
			});

			ui.menu_button("Options", |ui| {
				if options.light_mode {
					if ui.button("Switch to Dark Mode").clicked() {
						options.light_mode = false;
						ctx.set_visuals(Visuals::dark());
						ui.close_menu();
					}
				} else if ui.button("Switch to Light Mode").clicked() {
					options.light_mode = true;
					ctx.set_visuals(Visuals::light());
					ui.close_menu();
				}
			});
		});

		if let Some(dialog) = &mut file_tab.open_file_dialog {
			if dialog.show(ctx).selected() {
				if let Some(file) = dialog.path() {
					match fs::read_to_string(&file) {
						Ok(text) => {
							let file_name = file.to_string_lossy();

							macro_rules! try_these {
								($($type:ident,)+$(,)?) => {
									$(
										if file_name.contains(concat!(".", stringify!($type))) {
											paste! {
												match [<$type:camel Editor>]::new(&file, &text) {
													Ok(editor) => {
														result = Ok(Some(Box::new(editor)));
													}
													Err(msg) => {
														result = Err(Parse(msg));
													}
												}
											}
										} else
									)+
									{
										result = Err(UnknownFormat);
									}
								};
							}

							try_these!(
								item, class,
								// This should always be last because it never fails.
								toml,
							);

							file_tab.opened_file = Some(file.to_path_buf());
						}
						Err(msg) => {
							result = Err(Open(msg));
						}
					}
				}
				file_tab.open_file_dialog = None;
				ui.close_menu();
			}
		}
	});

	result
}
