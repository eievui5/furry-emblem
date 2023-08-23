use bevy::prelude::*;
use bevy::render::camera::*;
use bevy::render::render_resource::*;
use bevy::render::view::RenderLayers;
use fe_engine::input;
use fe_engine::module;

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
		.add_plugins(
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
		)
		.add_systems(Startup, add_camera)
		.add_systems(Update, rotate_cursor)
		.run();
}

fn rotate_cursor(time: Res<Time>, mut cursors: Query<&mut Transform, With<Cursor>>) {
	for mut i in &mut cursors {
		i.rotation = Quat::from_rotation_y((time.startup().elapsed().as_millis() as f32) / 300.0);
	}
}

fn add_camera(
	mut commands: Commands,
	mut assets: ResMut<Assets<Image>>,
	asset_server: Res<AssetServer>,
) {
	let size = Extent3d {
		width: WINDOW_SIZE.x,
		height: WINDOW_SIZE.y,
		..Default::default()
	};

	// This is the texture that will be rendered to.
	let mut image = Image {
		texture_descriptor: TextureDescriptor {
			label: None,
			size,
			dimension: TextureDimension::D2,
			format: TextureFormat::Bgra8UnormSrgb,
			mip_level_count: 1,
			sample_count: 1,
			usage: TextureUsages::TEXTURE_BINDING
				| TextureUsages::COPY_DST
				| TextureUsages::RENDER_ATTACHMENT,
			view_formats: &[],
		},
		..default()
	};

	// fill image.data with zeroes
	image.resize(size);

	let image_handle = assets.add(image);
	let scale_up_layer = RenderLayers::layer(1);

	commands.spawn((Camera2dBundle {
		camera: Camera {
			viewport: Some(Viewport {
				// TODO: adjust this to match the size of the window.
				physical_position: UVec2::ZERO,
				physical_size: WINDOW_SIZE,
				..default()
			}),
			target: RenderTarget::Image(image_handle.clone()),
			..default()
		},
		..default()
	},));
	commands.spawn((
		Cursor,
		SpriteBundle {
			texture: asset_server.load("../example-game/items/icons/storm-sword.png"),
			..Default::default()
		},
	));
	//commands.spawn((SpriteBundle {
	//	texture: asset_server.load("icon.png"),
	//	..Default::default()
	//},));

	commands.spawn((Camera2dBundle::default(), scale_up_layer));
	commands.spawn((
		SpriteBundle {
			texture: image_handle,
			transform: Transform {
				scale: Vec3 {
					x: 20.0,
					y: 20.0,
					..Default::default()
				},
				..Default::default()
			},
			..Default::default()
		},
		scale_up_layer,
	));
}
