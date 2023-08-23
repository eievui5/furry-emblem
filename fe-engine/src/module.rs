use std::fs;
use std::path::PathBuf;

pub use fe_data::Module;

fn try_load_module(path: PathBuf) -> anyhow::Result<Option<Module>> {
	let info = path.join("fe-project.toml");
	if info.exists() {
		let mut project = toml::from_str::<Module>(&fs::read_to_string(&info)?)?;
		project.populate(path);
		Ok(Some(project))
	} else {
		Ok(None)
	}
}

/// Loads all modules found the current working directory (not recursive).
///
/// # Errors
///
/// Fails if the module could not be opened or parsed.
#[must_use]
pub fn load_all() -> (Vec<Module>, Vec<String>) {
	let mut projects = Vec::new();
	let mut errors = Vec::new();

	if let Ok(mut dir) = fs::read_dir(".") {
		while let Some(Ok(entry)) = dir.next() {
			match try_load_module(entry.path()) {
				Ok(Some(module)) => projects.push(module),
				Ok(None) => {}
				Err(msg) => {
					// These errors should not halt the search,
					// but our caller will want to be aware of them.
					errors.push(format!("Failed to open {}: {msg}", entry.path().display()));
				}
			}
		}
	}

	(projects, errors)
}

#[must_use]
pub fn get_primary<'a>(modules: &'a [Module]) -> Option<&'a Module> {
	let mut result: Option<&'a Module> = None;
	for i in modules {
		if i.primary {
			if let Some(result) = result {
				log::warn!(
					"Multiple primary modules are loaded. Defaulting to {}",
					result.name
				);
				// Stop looking after at least one is found; there's no point in spamming the log.
				break;
			}
			result = Some(i);
		}
	}
	result
}
