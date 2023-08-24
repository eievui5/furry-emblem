use bevy::prelude::*;
use bevy::render::camera::*;
use bevy::render::render_resource::*;
use bevy::render::view::RenderLayers;

#[derive(Component)]
struct PixelCanvas;

/// A border shown around a pixel-perfect canvas layer.
/// Automatically scales to fit as much as possible and fill the screen.
#[derive(Component)]
pub struct FullscreenBorder(pub Handle<Image>);

/// Creates a pixel-perfect canvas layer.
pub struct PixelPerfectCanvas<const X: u32, const Y: u32>;

impl<const X: u32, const Y: u32> Plugin for PixelPerfectCanvas<X, Y> {
	fn build(&self, app: &mut App) {
		let resize_canvas =
			|windows: Query<&Window>, mut canvases: Query<&mut Transform, With<PixelCanvas>>| {
				let window = windows.single();
				let (x, y) = (window.resolution.width(), window.resolution.height());

				let x = (x / (X as f32)).floor().max(1.0);
				let y = (y / (Y as f32)).floor().max(1.0);
				let scale = x.min(y);

				for mut i in &mut canvases {
					i.scale.x = scale;
					i.scale.y = scale;
				}
			};

		let add_camera = |mut commands: Commands,
		                  mut assets: ResMut<Assets<Image>>,
		                  asset_server: Res<AssetServer>| {
			let size = Extent3d {
				width: X,
				height: Y,
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

			commands.spawn((Camera2dBundle {
				camera: Camera {
					viewport: Some(Viewport {
						// TODO: adjust this to match the size of the window.
						physical_position: UVec2::ZERO,
						physical_size: UVec2::new(X, Y),
						..default()
					}),
					target: RenderTarget::Image(image_handle.clone()),
					..default()
				},
				..default()
			},));

			let background_texture = asset_server.load("fe-engine/background.png");

			// Scaler layer
			// Projects the worldspace onto a statically sized texture.
			// This allows for proper pixel scaling without breaking the grid effect.
			let scale_up_layer = RenderLayers::layer(1);
			commands.spawn((Camera2dBundle::default(), scale_up_layer));
			commands.spawn((
				FullscreenBorder(background_texture.clone_weak()),
				SpriteBundle {
					texture: background_texture,
					transform: Transform {
						translation: Vec3 {
							z: -1.0,
							..Default::default()
						},
						..Default::default()
					},
					..Default::default()
				},
				scale_up_layer,
			));
			commands.spawn((
				SpriteBundle {
					texture: image_handle,
					..Default::default()
				},
				PixelCanvas,
				scale_up_layer,
			));
		};

		app.add_systems(Startup, add_camera)
			.add_systems(Update, (resize_canvas, resize_border));
	}
}

fn resize_border(
	windows: Query<&Window>,
	mut canvases: Query<(&mut Transform, &FullscreenBorder)>,
	assets: Res<Assets<Image>>,
) {
	let window = windows.single();
	let (x, y) = (window.resolution.width(), window.resolution.height());

	for (mut trans, border) in &mut canvases {
		let Some(image) = assets.get(&border.0) else {
			continue;
		};
		let x = x / (image.texture_descriptor.size.width as f32);
		let y = y / (image.texture_descriptor.size.height as f32);
		let scale = x.max(y);
		trans.scale.x = scale;
		trans.scale.y = scale;
	}
}
