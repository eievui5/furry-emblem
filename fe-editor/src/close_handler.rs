use egui::*;

#[derive(Default)]
pub struct CloseHandler {
	pub visible: bool,
	pub force_close: bool,
}

pub enum CloseHandlerResponse {
	SaveAndExit,
	Exit,
	Cancel,
}

impl CloseHandler {
	pub fn show(&mut self, ctx: &Context) -> Option<CloseHandlerResponse> {
		use CloseHandlerResponse::*;

		let mut response = None;

		Window::new("Exit?")
			.open(&mut self.visible)
			.show(ctx, |ui| {
				ui.heading("You have unsaved changes.");
				ui.label("Are you sure you want to exit?");
				ui.separator();
				if ui.button("Save all and exit").clicked() {
					response = Some(SaveAndExit);
				}
				if ui.button("Exit without saving").clicked() {
					response = Some(Exit);
				}
				if ui.button("Do not exit").clicked() {
					response = Some(Cancel);
				}
			});

		response
	}
}
