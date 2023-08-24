use bevy::prelude::*;
use fe_engine::input;
use fe_engine::module;
use fe_engine::ppcanvas::PixelPerfectCanvas;

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

#[derive(Component)]
struct Cursor;

fn main() {
	let (modules, module_errors) = module::load_all();

	for msg in module_errors {
		log::error!("{msg}");
	}

	for module in &modules {
		log::info!("Loaded module: {}", module.name);
	}

	let primary_module = module::get_primary(&modules);
	if let Some(primary_module) = primary_module {
		if let Some(_icon) = &primary_module.icon {
			//if let Err(msg) = set_icon(&window, icon.clone()) {
			//	log::error!("Failed to use module icon: {msg}");
			//}
		}
	}

	log::info!("Engine Initialized.");

	let _input_map = input::Map::from([
		("Ui:Move Up", input::Key),
		("Ui:Move Down", input::Key),
		("Ui:Move Left", input::Key),
		("Ui:Move Right", input::Key),
	]);

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
					asset_folder: String::from("./"),
					..Default::default()
				}),
			PixelPerfectCanvas::<{ WINDOW_SIZE.x }, { WINDOW_SIZE.y }>,
		))
		.add_systems(Startup, spawn_cursor)
		.add_systems(Update, rotate_cursor)
		.run();
}

fn rotate_cursor(time: Res<Time>, mut cursors: Query<&mut Transform, With<Cursor>>) {
	for mut i in &mut cursors {
		i.rotation = Quat::from_rotation_y((time.startup().elapsed().as_millis() as f32) / 300.0);
	}
}

fn spawn_cursor(mut commands: Commands, asset_server: Res<AssetServer>) {
	commands.spawn((
		Cursor,
		SpriteBundle {
			texture: asset_server.load("../example-game/items/icons/storm-sword.png"),
			..Default::default()
		},
	));
}
