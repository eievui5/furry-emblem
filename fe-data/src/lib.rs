use serde::{Deserialize, Serialize};
use std::fmt;
use std::num::NonZeroU32;

#[cfg(feature = "stata")]
use {
	quote::quote,
	stata::{Resource, ToStatic, TokenStream},
};

#[cfg_attr(feature = "stata", derive(Resource))]
#[derive(Clone, Default, Debug, Deserialize, Eq, PartialEq, Serialize)]
#[serde(default)]
pub struct Stats {
	// Growth stats
	// Increase upon level up.
	pub hp: i32,
	pub power: i32,
	pub defense: i32,
	pub resistance: i32,
	pub dexterity: i32,
	// Static stats
	// Only increase upon promotion.
	pub movement: i32,
	pub constitution: i32,
	pub reflexes: i32,
}

#[cfg_attr(feature = "stata", derive(Resource))]
#[derive(Clone, Default, Debug, Deserialize, Eq, PartialEq, Serialize)]
#[serde(default)]
pub struct Class {
	pub name: String,

	/// Base stats for a given class.
	/// Individual characters should offset this to provide some more unique spreads.
	pub bases: Stats,
	/// Base growths for a given class.
	/// Like `stats`, these are offset by characters.
	pub growths: Stats,

	// Movement skills
	/// Unit can move after performing any action other than attacking.
	pub canter: bool,
	/// Unit can push another unit one tile forward.
	pub shove: bool,
	/// Unit can move through enemies.
	pub pass: bool,
	/// Unit can move to the opposite side of another unit.
	pub leap: bool,
	/// Unit can move another unit to their opposite side.
	pub pull: bool,

	// Bonus skills
	/// Unit can only be defeated when at exactly 1 hp.
	/// A critical hit ignores this effect.
	pub focus: bool,
	/// Boosts critical hit rate by 20%.
	pub crit_boost: bool,
	/// Allows a unit to move after attacking.
	pub battle_canter: bool,
	/// Unit can rescue any unit regardless of constitution.
	pub ferry: bool,
	/// Enemy units cannot enter 1-tile gaps around this unit.
	pub zone_of_control: bool,

	// Weaknesses
	pub armored: bool,
	pub flying: bool,
	pub agile: bool,
}

#[cfg_attr(feature = "stata", derive(Resource))]
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

#[cfg(feature = "stata")]
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

#[cfg(feature = "stata")]
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

#[cfg_attr(feature = "stata", derive(Resource))]
#[derive(Clone, Default, Debug, Deserialize, Eq, PartialEq, Serialize)]
#[serde(default)]
pub struct Item {
	pub name: String,
	pub value: Option<NonZeroU32>,
	#[serde(rename = "type")]
	pub ty: ItemType,
}
