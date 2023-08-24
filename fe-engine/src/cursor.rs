use bevy::prelude::*;
use leafwing_input_manager::prelude::*;

#[derive(Actionlike, PartialEq, Eq, Clone, Copy, Hash, Debug, Reflect)]
pub enum UiAction {
	Up,
	Down,
	Left,
	Right,
	Select,
	Back,
}

#[derive(Component)]
pub struct Cursor;

pub fn rotate(time: Res<Time>, mut cursors: Query<&mut Transform, With<Cursor>>) {
	for mut i in &mut cursors {
		i.rotation = Quat::from_rotation_y((time.startup().elapsed().as_millis() as f32) / 300.0);
	}
}

pub fn spawn(mut commands: Commands, asset_server: Res<AssetServer>) {
	let texture = asset_server.load("example-game/cursor.png");

	commands
		.spawn((
			InputManagerBundle {
				action_state: ActionState::default(),
				input_map: InputMap::new([
					(KeyCode::Up, UiAction::Up),
					(KeyCode::Down, UiAction::Down),
					(KeyCode::Left, UiAction::Left),
					(KeyCode::Right, UiAction::Right),
					(KeyCode::Return, UiAction::Select),
					(KeyCode::Space, UiAction::Select),
				]),
			},
			Transform::default(),
			GlobalTransform::default(),
			Visibility::default(),
			ComputedVisibility::default(),
			Cursor,
		))
		.with_children(|parent| {
			parent.spawn(SpriteBundle {
				sprite: Sprite {
					flip_y: true,
					..Default::default()
				},
				transform: Transform {
					translation: Vec3 {
						x: -8.0,
						y: -8.0,
						..Default::default()
					},
					..Default::default()
				},
				texture: texture.clone(),
				..Default::default()
			});
			parent.spawn(SpriteBundle {
				sprite: Sprite {
					flip_x: true,
					..Default::default()
				},
				transform: Transform {
					translation: Vec3 {
						x: 8.0,
						y: 8.0,
						..Default::default()
					},
					..Default::default()
				},
				texture: texture.clone(),
				..Default::default()
			});
			parent.spawn(SpriteBundle {
				transform: Transform {
					translation: Vec3 {
						x: -8.0,
						y: 8.0,
						..Default::default()
					},
					..Default::default()
				},
				texture: texture.clone(),
				..Default::default()
			});
			parent.spawn(SpriteBundle {
				sprite: Sprite {
					flip_y: true,
					flip_x: true,
					..Default::default()
				},
				transform: Transform {
					translation: Vec3 {
						x: 8.0,
						y: -8.0,
						..Default::default()
					},
					..Default::default()
				},
				texture: texture.clone(),
				..Default::default()
			});
		});
}

pub fn movement(mut cursors: Query<(&mut Transform, &ActionState<UiAction>)>) {
	const TILE_SIZE: f32 = 16.0;

	for (mut transform, action) in &mut cursors {
		transform.translation.x += TILE_SIZE
			* if action.just_pressed(UiAction::Left) {
				-1.0
			} else if action.just_pressed(UiAction::Right) {
				1.0
			} else {
				0.0
			};
		transform.translation.y += TILE_SIZE
			* if action.just_pressed(UiAction::Up) {
				1.0
			} else if action.just_pressed(UiAction::Down) {
				-1.0
			} else {
				0.0
			};
	}
}
