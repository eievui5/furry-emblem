use serde::{Deserialize, Serialize};

pub type Tile = u8;

#[cfg_attr(feature = "sucrose", derive(Resource))]
#[derive(Clone, Default, Debug, Deserialize, Eq, PartialEq, Serialize)]
#[serde(default)]
pub struct Map {
	pub name: String,
	pub width: u32,
	pub height: u32,
	pub tiles: Vec<Tile>,
}
