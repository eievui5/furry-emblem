use crate::editors::*;
use crate::error;
use crate::{EditorApp, FILE_TYPE_STRINGS};
use egui::*;
use egui_toast::Toasts;
use paste::paste;
use std::fs;

pub fn show(app: &mut EditorApp, toasts: &mut Toasts, ctx: &egui::Context) {
	TopBottomPanel::top("Menu Bar").show(ctx, |ui| {
		ui.horizontal(|ui| {
			ui.menu_button("File", |ui| {
				ui.menu_button("New", |ui| {
					for (i, name) in FILE_TYPE_STRINGS.iter().enumerate() {
						if ui.button(*name).clicked() {
							app.new_file_window.open(i);
							ui.close_menu();
						}
					}
				});
				if ui.button("Open").clicked() {
					let mut dialog = egui_file::FileDialog::save_file(app.opened_file.clone());
					dialog.open();
					app.open_file_dialog = Some(dialog);
					ui.close_menu();
				}
			});

			ui.menu_button("Options", |ui| {
				if app.light_mode {
					if ui.button("Switch to Dark Mode").clicked() {
						app.light_mode = false;
						ctx.set_visuals(Visuals::dark());
						ui.close_menu();
					}
				} else if ui.button("Switch to Light Mode").clicked() {
					app.light_mode = true;
					ctx.set_visuals(Visuals::light());
					ui.close_menu();
				}
			});
		});

		if let Some(dialog) = &mut app.open_file_dialog {
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
													Ok(editor) => app.editors.push(Box::new(editor)),
													Err(msg) => {
														toasts.add(error(format!("Failed to parse {}:\n{msg}", file.display()).into()));
													}
												}
											}
										} else
									)+
									{
										toasts.add(error(format!("{} did not match any expected formats", file.display()).into()));
									}
								};
							}

							try_these!(
								item, class,
								// This should always be last because it never fails.
								toml,
							);

							app.opened_file = Some(file.to_path_buf());
						}
						Err(msg) => {
							toasts.add(error(
								format!("Failed to open {}:\n{msg}", file.display()).into(),
							));
						}
					}
				}
				app.open_file_dialog = None;
				ui.close_menu();
			}
		}
	});
}
