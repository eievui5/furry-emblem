use serde::{Deserialize, Serialize};
use std::path::PathBuf;

#[cfg(feature = "sucrose")]
use sucrose::{quote, ToStatic, TokenStream};

#[derive(Clone, Default, Debug, Deserialize, Eq, PartialEq, Serialize)]
/// Image file wrapper.
/// Gets converted to binary graphics when loaded by sucrose.
pub struct Image {
	pub path: PathBuf,
}

#[cfg(feature = "sucrose")]
impl ToStatic for Image {
	fn static_type() -> TokenStream {
		quote!(&'static str)
	}
	fn static_value(&self) -> TokenStream {
		let path = self.path.to_string_lossy();
		quote!(#path)
	}
}
