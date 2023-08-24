use bevy::prelude::*;
use bevy::window::WindowMode;
use fe_engine::cursor;
use fe_engine::module;
use fe_engine::ppcanvas::PixelPerfectCanvas;
use leafwing_input_manager::prelude::*;

const DEFAULT_TITLE: &str = "Furry Emblem Engine";
const WINDOW_SIZE: UVec2 = UVec2::new(240, 160);

/*
fn set_icon(window: &PistonWindow, icon: DynamicImage) -> anyhow::Result<()> {
	// TODO: embed the default icon into the binary (like fe-editor) so that it is always available.
	// TODO: allow the icon to be overwritten by projects.
	// `piston_window` very inconsiderately has a module named `image` so we need this root syntax.
	let width = icon.width();
	let height = icon.height();
	let icon = icon.to_rgba8().into_vec();
	let icon = Icon::from_rgba(icon, width, height)?;
	// So many wrapper types...
	window.window.window.set_window_icon(Some(icon));
	Ok(())
}

fn open_icon(window: &PistonWindow, path: impl AsRef<Path>) -> anyhow::Result<()> {
	let icon = image::open(path.as_ref())?;
	set_icon(window, icon)
}
*/

fn main() {
	let modules = module::load_all();

	for module in &modules {
		info!("Loaded module: {}", module.name);
	}

	let primary_module = module::get_primary(&modules);
	if let Some(primary_module) = primary_module {
		if let Some(_icon) = &primary_module.icon {
			//if let Err(msg) = set_icon(&window, icon.clone()) {
			//	error!("Failed to use module icon: {msg}");
			//}
		}
	}

	info!("Engine Initialized.");

	App::new()
		.add_plugins((
			DefaultPlugins
				.set(WindowPlugin {
					primary_window: Some(Window {
						title: DEFAULT_TITLE.to_string(),
						..Default::default()
					}),
					..Default::default()
				})
				.set(ImagePlugin::default_nearest())
				.set(AssetPlugin {
					asset_folder: String::from("../"),
					..Default::default()
				}),
			PixelPerfectCanvas::<{ WINDOW_SIZE.x }, { WINDOW_SIZE.y }>,
			InputManagerPlugin::<cursor::UiAction>::default(),
		))
		.add_systems(Startup, (cursor::spawn, spawn_unit))
		.add_systems(Update, (cursor::movement, cursor::rotate))
		.add_systems(Update, fullscreen)
		.run();
}

fn fullscreen(mut windows: Query<&mut Window>, keys: Res<Input<KeyCode>>) {
	// TODO: Use an event for this.
	if keys.just_pressed(KeyCode::F11) {
		let mut window = windows.single_mut();
		window.mode = if window.mode == WindowMode::Fullscreen {
			WindowMode::Windowed
		} else {
			WindowMode::Fullscreen
		};
	}
}

fn spawn_unit(
	mut commands: Commands,
	asset_server: Res<AssetServer>,
	mut texture_atlases: ResMut<Assets<TextureAtlas>>,
) {
	let texture_handle = asset_server.load("example-game/classes/icons/cat.png");
	let texture_atlas =
		TextureAtlas::from_grid(texture_handle, Vec2::new(16.0, 16.0), 6, 1, None, None);
	let texture_atlas = texture_atlases.add(texture_atlas);

	commands.spawn(SpriteSheetBundle {
		texture_atlas,
		sprite: TextureAtlasSprite::new(0),
		..Default::default()
	});
}
