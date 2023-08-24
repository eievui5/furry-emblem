use crate::containers::Image;
use crate::{ClassReference, Stats};
use serde::{Deserialize, Serialize};

/// Determines bonuses given by supports.
///
/// By default, this gives a bonus to a unit's growths,
/// But in the future other options may be given.
#[cfg_attr(feature = "sucrose", derive(sucrose::Resource))]
#[derive(Clone, Default, Debug, Deserialize, Eq, PartialEq, Serialize)]
#[serde(default)]
pub struct Personality {
	pub name: String,
	pub icon: Image,
	// TODO: Consider adding a config option to change the impact of personality/supports.
	pub growths_bonus: Stats,
}

make_reference!(personality::Personality => PersonalityReference);

/// Everything that makes up a (named) character.
///
/// There may be other variants of this type with less information, to imply certain fields and variations.
/// For example, enemy units are less fleshed out and need less defined.
/// These base types are used to create a true unit at runtime.
///
/// Scripting can be used to associate or modify this information.
/// For example, an inventory can be attached, level ups can be applied,
/// or modifications can be made to its stats.
#[cfg_attr(feature = "sucrose", derive(sucrose::Resource))]
#[derive(Clone, Default, Debug, Deserialize, Eq, PartialEq, Serialize)]
#[serde(default)]
pub struct Unit {
	pub name: String,
	pub description: Option<String>,
	/// Unit's affiliation, such as a part or team.
	/// If absent, class name will be displayed instead.
	pub affiliation: Option<String>,
	pub class: ClassReference,
	// Bases offset. Applied on top of class bases.
	pub bases: Stats,
	// Growths offset. Applied on top of class growths.
	pub growths: Stats,
	pub personality: Option<PersonalityReference>,
}
