use crate::containers::Image;
use crate::{make_reference, Stats};
use serde::{Deserialize, Serialize};

make_reference!(classes::Class => ClassReference);

#[cfg_attr(feature = "sucrose", derive(sucrose::Resource))]
#[derive(Clone, Default, Debug, Deserialize, Eq, PartialEq, Serialize)]
#[serde(default)]
pub struct Class {
	pub name: String,
	pub description: String,
	pub icon: Image,

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
