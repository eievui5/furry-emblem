use bevy::prelude::*;
use bevy::window::WindowMode;
use bevy_ecs_tilemap::prelude::*;
use fe_engine::cursor;
use fe_engine::module;
use fe_engine::ppcanvas::PixelPerfectCanvas;
use leafwing_input_manager::prelude::*;
use mlua::chunk;
use mlua::prelude::*;

const DEFAULT_TITLE: &str = "Furry Emblem Engine";
const WINDOW_SIZE: UVec2 = UVec2::new(240, 160);

/*
fn set_icon(window: &PistonWindow, icon: DynamicImage) -> anyhow::Result<()> {
	// TODO: embed the default icon into the binary (like fe-editor) so that it is always available.
	// TODO: allow the icon to be overwritten by projects.
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
			TilemapPlugin,
		))
		.add_systems(
			Startup,
			(cursor::spawn, spawn_unit, startup, luau_scripting),
		)
		.add_systems(Update, (cursor::movement, cursor::rotate))
		.add_systems(Update, fullscreen)
		.run();
}

fn fullscreen(mut windows: Query<&mut Window>, keys: Res<Input<KeyCode>>) {
	// TODO: Use an event for this.
	if keys.just_pressed(KeyCode::F11) {
		// Borderless is th eonly fullscreen mode that works on Wayland.
		const FULLSCREEN: WindowMode = WindowMode::BorderlessFullscreen;

		let mut window = windows.single_mut();
		window.mode = if window.mode == FULLSCREEN {
			WindowMode::Windowed
		} else {
			FULLSCREEN
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

fn startup(
	mut commands: Commands,
	asset_server: Res<AssetServer>,
	array_texture_loader: Res<ArrayTextureLoader>,
) {
	let texture_handle: Handle<Image> = asset_server.load("example-game/tileset.png");
	let map_size = TilemapSize { x: 15, y: 11 };
	let tilemap_entity = commands.spawn_empty().id();
	let mut tile_storage = TileStorage::empty(map_size);

	for x in 0..map_size.x {
		for y in 0..map_size.y {
			let tile_pos = TilePos { x, y };
			let tile_entity = commands
				.spawn(TileBundle {
					position: tile_pos,
					tilemap_id: TilemapId(tilemap_entity),
					..Default::default()
				})
				.id();
			tile_storage.set(&tile_pos, tile_entity);
		}
	}

	let tile_size = TilemapTileSize { x: 16.0, y: 16.0 };
	let grid_size = tile_size.into();
	let map_type = TilemapType::default();
	let mut transform = get_tilemap_center_transform(&map_size, &grid_size, &map_type, 0.0);
	transform.translation.z -= 16.0;

	commands.entity(tilemap_entity).insert(TilemapBundle {
		grid_size,
		map_type,
		size: map_size,
		storage: tile_storage,
		texture: TilemapTexture::Single(texture_handle.clone()),
		tile_size,
		transform,
		..Default::default()
	});

	// Add atlas to array texture loader so it's preprocessed before we need to use it.
	// Only used when the atlas feature is off and we are using array textures.
	array_texture_loader.add(TilemapArrayTexture {
		texture: TilemapTexture::Single(texture_handle),
		tile_size,
		..Default::default()
	});
}

fn luau_scripting() {
	let lua = Lua::new();
	let globals = lua.globals();

	#[derive(Copy, Clone, Debug, FromLua)]
	enum Event {
		Immediately,
		WaitPrint,
		WaitMove,
	}

	impl mlua::UserData for Event {}

	globals
		.set(
			"Event",
			lua.create_table_from([
				("Immediately", Event::Immediately),
				("WaitPrint", Event::WaitPrint),
				("WaitMove", Event::WaitMove),
			])
			.unwrap(),
		)
		.unwrap();

	globals
		.set(
			"say",
			lua.create_function(move |_, s: String| {
				println!("{s}");
				Ok(Event::WaitPrint)
			})
			.unwrap(),
		)
		.unwrap();

	globals
		.set(
			"move",
			lua.create_function(move |_, (x, y): (i32, i32)| {
				println!("- Moved by ({x}, {y})");
				Ok(Event::WaitMove)
			})
			.unwrap(),
		)
		.unwrap();

	let chunk = lua.load(chunk! {
		local yield = coroutine.yield

		local function onEvent()
			yield(say("Hello!"))
			say("I'm moving")
			move(0, 2)
			// Explicit yield type.
			// You only need this when multiple events have been started and you only want to wait for one.
			yield(Event.WaitMove)
			// Alternatively, wait for both to complete:
			yield(say("Moving again!"))
		end

		signal = {}
		signal.interact = coroutine.create(onEvent)
	});
	chunk.exec().unwrap();
	let event = globals
		.get::<&str, mlua::Table>("signal")
		.and_then(|t| t.get::<&str, mlua::Thread>("interact"));

	if let Ok(event) = event {
		while event.status() == LuaThreadStatus::Resumable {
			if let Ok(Some(event)) = event.resume::<(), Option<Event>>(()) {
				println!("Thread is yielding until: {event:?}",);
			} else {
				println!("Thread exited without requesting event.");
			}
		}
		println!("Thread complete.");
	}
}
