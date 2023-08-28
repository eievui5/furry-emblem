use serde::{Deserialize, Serialize};
use std::path::PathBuf;

pub mod containers;

#[macro_export]
macro_rules! make_reference {
	($container:ident :: $target:ident => $newtype:ident) => {
		#[derive(Clone, Default, Debug, Deserialize, Eq, PartialEq, Serialize)]
		#[serde(default)]
		#[doc = concat!("Wrapper type around ", stringify!($target), ", producing a `&'static` reference when make static.")]
		pub struct $newtype {
			identifier: String,
		}

		#[cfg(feature = "sucrose")]
		impl sucrose::ToStatic for $newtype {
			fn static_type() -> ::sucrose::TokenStream {
				sucrose::quote!(&'static $target)
			}
			fn static_value(&self) -> ::sucrose::TokenStream {
				let data = ::sucrose::proc_macro2::Ident::new(&self.identifier, ::sucrose::proc_macro2::Span::call_site());
				sucrose::quote!(&super::$container::#data)
			}
		}
	};
}

/// Define a module and publically import all of its members.
macro_rules! import {
	($name:ident $(,)?) => {
		mod $name;
		pub use $name:: *;
	};
	{$name:ident, $($remaining:ident),+ $(,)?} => {
		import!($name);
		import!($($remaining),+);
	};
}

import! {
	items,
	map,
	class,
	stats,
	unit,
}

#[derive(Clone, Default, Debug, Deserialize, Serialize)]
pub struct Module {
	// Required fields
	pub name: String,

	// Optional fields.
	#[serde(default)]
	pub primary: bool,
	#[serde(default)]
	pub icon_path: Option<PathBuf>,

	// Runtime info
	#[serde(skip)]
	#[cfg(feature = "runtime")]
	pub path: PathBuf,
	#[serde(skip)]
	#[cfg(feature = "runtime")]
	pub icon: Option<image::DynamicImage>,
}

impl Eq for Module {}

impl PartialEq for Module {
	fn eq(&self, other: &Module) -> bool {
		macro_rules! compare {
			($($ident:ident),+) => {
				$(
					self.$ident == other.$ident
				)&&+
			}
		}
		compare!(name, primary, icon_path)
	}
}

#[cfg(feature = "runtime")]
impl Module {
	/// Fills runtime fields according to inputs.
	pub fn populate(&mut self, path: PathBuf) {
		use std::path::Path;

		let icon_path = path.join(self.icon_path.as_ref().map_or(Path::new("icon.png"), |p| p));
		match image::open(&icon_path) {
			Ok(icon) => self.icon = Some(icon),
			Err(msg) => {
				// If the user didn't ask to load an icon, they won't care about this error.
				if self.icon_path.is_some() {
					log::error!("Failed to load custom icon: {}: {msg}", icon_path.display());
				}
			}
		}
		self.path = path;
	}
}
