use crate::containers::Image;
use grid::Grid;
use serde::{Deserialize, Serialize};

/// Integer representing tile IDs.
/// Implicitly determines tile limit;
/// increase if more are needed.
pub type Tile = u8;

#[cfg_attr(feature = "sucrose", derive(Resource))]
#[derive(Clone, Debug, Deserialize, Eq, PartialEq, Serialize)]
#[serde(default)]
pub struct Map {
	pub name: String,
	pub tiles: Grid<Tile>,
	pub tileset: TilesetReference,
}

impl Default for Map {
	fn default() -> Self {
		Self {
			name: String::new(),
			tiles: Grid::new(10, 15),
			tileset: TilesetReference::default(),
		}
	}
}

#[cfg_attr(feature = "sucrose", derive(Resource))]
#[derive(Clone, Default, Debug, Deserialize, Eq, PartialEq, Serialize)]
#[serde(default)]
pub struct TileInfo {
	/// Name of the terrain type used by this tile.
	///
	/// `None` represents a clear terrain, with no benefits or penalties.
	pub terrain: Option<String>,
	/// X coordinate of this tile within the source map.
	/// This is multiplied by the map size so that it is always valid.
	pub x: u32,
	/// Y coordinate of this tile within the source map.
	/// This is multiplied by the map size so that it is always valid.
	pub y: u32,
}

make_reference!(tilesets::Tileset => TilesetReference);

#[derive(Clone, Default, Debug, Deserialize, Eq, PartialEq, Serialize)]
pub enum TileEntry {
	Tile(TileInfo),
	#[default]
	Gap,
}

#[cfg(feature = "sucrose")]
impl ToStatic for TileEntry {
	fn static_type() -> TokenStream {
		quote!(TileEntry)
	}

	fn static_value(&self) -> TokenStream {
		match self {
			TileEntry::Tile(info) => {
				let info = info.static_value();
				quote!(TileEntry::Tile(#info))
			}
			TileEntry::Gap => quote!(TileEntry::Gap),
		}
	}
}

#[cfg(feature = "sucrose")]
impl Resource for TileEntry {
	fn static_struct() -> TokenStream {
		quote! {
			#[derive(Clone, Debug, Default)]
			pub enum TileEntry {
				Tile(TileInfo),
				#[default]
				Gap,
			}
		}
	}
}

#[cfg_attr(feature = "sucrose", derive(Resource))]
#[derive(Clone, Debug, Deserialize, Eq, PartialEq, Serialize)]
#[serde(default)]
/// Contains an image and info describing the locations and meanings of tiles.
///
/// Tiles can be assigned IDs independently of their physical positions,
/// to maintain compatibility while still using compact IDs
pub struct Tileset {
	pub name: String,
	/// Source image for tile contents.
	pub texture: Image,
	pub tile_width: u32,
	/// Describes the characteristics of each possible ID.
	///
	/// There may be gaps;
	/// this allows changes to be made to the tileset while keeping old tiles in place.
	/// Any tiles assigned to missing IDs should be reassigned to some placeholder.
	pub atlas: Vec<TileEntry>,
}

impl Default for Tileset {
	fn default() -> Self {
		Self {
			name: String::new(),
			texture: Image::default(),
			tile_width: 8,
			atlas: Vec::new(),
		}
	}
}
