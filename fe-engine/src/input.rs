use std::collections::HashMap;

// Dummy placeholder
#[derive(Copy, Clone, Debug, Eq, PartialEq, Hash)]
pub struct Key;

#[derive(Copy, Clone, Debug, Eq, PartialEq)]
struct KeyStateHistory {
	/// Set once the update loop sees a fresh press for the first time.
	/// The key state will decay on the next update if this is set.
	press_ack: bool,
	/// Set once the update loop sees a fresh release for the first time.
	/// The key state will decay on the next update if this is set.
	release_ack: bool,
	state: KeyState,
}

#[derive(Copy, Clone, Debug, Eq, PartialEq)]
pub enum KeyState {
	None,
	New,
	Held,
	Released,
}

impl KeyState {
	#[must_use]
	pub fn pressed(self) -> bool {
		matches!(self, KeyState::New | KeyState::Held)
	}

	#[must_use]
	pub fn just_pressed(self) -> bool {
		self == KeyState::New
	}

	#[must_use]
	pub fn just_released(self) -> bool {
		self == KeyState::Released
	}
}

#[derive(Clone, Default, Debug)]
pub struct Map {
	map: HashMap<String, Key>,
	state: HashMap<Key, KeyStateHistory>,
}

impl<const N: usize> From<[(&str, Key); N]> for Map {
	fn from(slice: [(&str, Key); N]) -> Self {
		let mut new = Self::default();
		for (k, v) in slice {
			new.register(k.to_owned(), v);
		}
		new
	}
}

impl Map {
	/// # Panics
	///
	/// Panics if a keymap is registered as using a given key but that key is not tracked.
	/// This should not happen unless the implementation of `[Map::register]` is bugged.
	pub fn get(&self, key: impl AsRef<str>) -> KeyState {
		let key = key.as_ref();
		self.map.get(key).map_or_else(
			|| {
				log::debug!("Couldn't find keymap: {key}");
				KeyState::None
			},
			|k| {
				self.state
					.get(k)
					.unwrap_or_else(|| {
						panic!("Keymap {key} tried to read {k:?} but it was not registered");
					})
					.state
			},
		)
	}

	pub fn press(&mut self, key: Key) {
		if let Some(key) = self.state.get_mut(&key) {
			key.state = KeyState::New;
		}
	}

	pub fn release(&mut self, key: Key) {
		if let Some(key) = self.state.get_mut(&key) {
			key.state = KeyState::Released;
		}
	}

	pub fn acknowledge(&mut self) {
		for state in self.state.values_mut() {
			match state.state {
				KeyState::New => {
					if state.press_ack {
						state.state = KeyState::Held;
					}
					state.press_ack ^= true;
				}
				KeyState::Released => {
					if state.release_ack {
						state.state = KeyState::None;
					}
					state.release_ack ^= true;
				}
				_ => {}
			}
		}
	}

	pub fn register(&mut self, identifier: String, key: Key) {
		self.map.insert(identifier, key);
		self.state.insert(
			key,
			KeyStateHistory {
				press_ack: false,
				release_ack: false,
				state: KeyState::None,
			},
		);
	}
}
