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
		impl ToStatic for $newtype {
			fn static_type() -> TokenStream {
				quote!(&'static $target)
			}
			fn static_value(&self) -> TokenStream {
				let data = proc_macro2::Ident::new(&self.identifier, proc_macro2::Span::call_site());
				quote!(&super::$container::#data)
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
	class,
	stats,
	unit,
}
