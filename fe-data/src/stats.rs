use serde::{Deserialize, Serialize};

#[cfg_attr(feature = "sucrose", derive(sucrose::Resource))]
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
