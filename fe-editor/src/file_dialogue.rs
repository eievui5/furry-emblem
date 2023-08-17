use pathdiff::diff_paths;
use std::path::{Path, PathBuf};
use std::thread;

#[derive(Default, Debug)]
pub struct FilePicker {
	handle: Option<thread::JoinHandle<Option<PathBuf>>>,
}

impl Clone for FilePicker {
	fn clone(&self) -> Self {
		Self { handle: None }
	}
}

impl FilePicker {
	pub fn open(&mut self) {
		if self.handle.is_some() {
			return;
		}
		self.handle = Some(thread::spawn(move || {
			rfd::FileDialog::new()
				.set_directory(
					Path::new("./")
						.canonicalize()
						.unwrap_or(PathBuf::from("./")),
				)
				.pick_file()
		}));
	}

	pub fn try_take_relative(&mut self, to: &Path) -> Option<PathBuf> {
		if let Some(handle) = &self.handle {
			if handle.is_finished() {
				if let Ok(Some(path)) = self.handle.take().unwrap().join() {
					let parent = to
						.parent()
						.unwrap_or(Path::new(""))
						.canonicalize()
						.unwrap_or(PathBuf::from(""));
					return Some(diff_paths(&path, parent).unwrap_or(path).to_path_buf());
				}
			}
		}
		None
	}
}
