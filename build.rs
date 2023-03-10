use evgfx::convert;
use fe_data::*;
use std::env;
use std::error::Error;
use std::fs;
use std::path::PathBuf;

fn convert_image(
	config: &convert::Config,
	input_path: &str,
	output_path: &PathBuf,
	palette_path: &PathBuf,
) -> Result<(), Box<dyn Error>> {
	println!("cargo:rerun-if-changed={input_path}");
	fs::create_dir_all(output_path.parent().unwrap())?;
	fs::create_dir_all(palette_path.parent().unwrap())?;

	let (palettes, tiles, _) = config.convert_image(input_path).unwrap();

	tiles.write_4bpp(output_path.to_str().unwrap()).unwrap();
	palettes.write_rgb555(palette_path.to_str().unwrap(), true).unwrap();

	Ok(())
}

macro_rules! make_image {
	($config:expr, $resource:expr) => {
		convert_image(
			$config,
			concat!("src/assets/", $resource, ".png"),
			&[
				&env::var("OUT_DIR")?,
				concat!("assets/", $resource, ".4bpp"),
			]
			.iter()
			.collect(),
			&[
				&env::var("OUT_DIR")?,
				concat!("assets/", $resource, ".pal"),
			]
			.iter()
			.collect(),
		)?;
	};
}

fn main() -> Result<(), Box<dyn Error>> {
	let config = convert::Config::new()
		.with_tilesize(16, 16)
		.with_transparency_color(0xFF, 0x00, 0xFF);

	make_image!(&config, "gfx/luvui");
	make_image!(&config, "gfx/tree_tiles");

	let config = convert::Config::new()
		.with_transparency_color(0xFF, 0x00, 0xFF);

	make_image!(&config, "gfx/cursor");

	let level = MapData::open("src/assets/maps/", "Debug Map".to_string())?;
	println!("cargo:rerun-if-changed=src/assets/maps/Debug Map.toml");
	let outpath: PathBuf = [&env::var("OUT_DIR")?, "assets/maps/", "Debug Map.rs"].iter().collect();
	fs::create_dir_all(outpath.parent().unwrap())?;
	fs::write(outpath, level.to_engine()?)?;

	Ok(())
}
