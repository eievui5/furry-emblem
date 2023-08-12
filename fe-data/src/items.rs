use crate::Image;
use serde::{Deserialize, Serialize};
use std::fmt;
use std::num::NonZeroU32;

#[cfg(feature = "sucrose")]
use {
	quote::quote,
	sucrose::{Resource, ToStatic, TokenStream},
};

#[derive(Clone, Default, Debug, Deserialize, Eq, PartialEq, Serialize)]
#[serde(default)]
pub struct ItemReference {
	identifier: String,
}

#[cfg(feature = "sucrose")]
impl ToStatic for ItemReference {
	fn static_type() -> TokenStream {
		quote!(&'static Item)
	}
	fn static_value(&self) -> TokenStream {
		let data = proc_macro2::Ident::new(&self.identifier, proc_macro2::Span::call_site());
		quote!(&super::items::#data)
	}
}

#[cfg_attr(feature = "sucrose", derive(Resource))]
#[derive(Clone, Default, Debug, Deserialize, Eq, PartialEq, Serialize)]
#[serde(default)]
pub struct WeaponItem {
	pub damage: u32,
	pub weight: u32,
	pub durability: u32,
}

#[derive(Clone, Default, Debug, Deserialize, Eq, PartialEq, Serialize)]
pub enum ItemType {
	// Does nothing.
	#[default]
	None,
	Weapon(WeaponItem),
}

#[cfg(feature = "sucrose")]
impl ToStatic for ItemType {
	fn static_type() -> TokenStream {
		quote!(ItemType)
	}
	fn static_value(&self) -> TokenStream {
		use ItemType::*;

		match self {
			None => quote!(ItemType::None),
			Weapon(item) => {
				let item = item.static_value();
				quote!(ItemType::Weapon(#item))
			}
		}
	}
}

#[cfg(feature = "sucrose")]
impl Resource for ItemType {
	fn static_struct() -> TokenStream {
		quote! {
			#[derive(Clone, Debug, Default)]
			pub enum ItemType {
				// Does nothing.
				#[default]
				None,
				Weapon(WeaponItem),
			}
		}
	}
}

impl fmt::Display for ItemType {
	fn fmt(&self, f: &mut fmt::Formatter<'_>) -> Result<(), fmt::Error> {
		use ItemType::*;
		write!(
			f,
			"{}",
			match self {
				None => "None",
				Weapon(..) => "Weapon",
			}
		)
	}
}

#[cfg_attr(feature = "sucrose", derive(Resource))]
#[derive(Clone, Default, Debug, Deserialize, Eq, PartialEq, Serialize)]
#[serde(default)]
pub struct Item {
	pub name: String,
	pub description: String,
	pub icon: Image,
	pub value: Option<NonZeroU32>,
	#[serde(rename = "type")]
	pub ty: ItemType,
}
