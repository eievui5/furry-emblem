#[derive(Copy, Clone, Debug, Eq, PartialEq)]
pub enum Cardinal {
	North,
	South,
	East,
	West,
}

impl Cardinal {
	#[must_use]
	pub fn as_point<I: From<i8>>(&self) -> (I, I) {
		use Cardinal::*;

		let (x, y) = match self {
			North => (0, -1),
			South => (0, 1),
			East => (-1, 0),
			West => (1, 0),
		};
		(x.into(), y.into())
	}
}

#[derive(Copy, Clone, Debug, Eq, PartialEq)]
pub enum Ordinal {
	North,
	Northeast,
	Northwest,
	South,
	Southeast,
	Southwest,
	East,
	West,
}

impl Ordinal {
	#[must_use]
	pub fn introduce(self, other: Self) -> Self {
		use Ordinal::*;

		macro_rules! one_of {
			{ $(($first:ident, $second:ident) => $result:ident),+ $(,)? } => {
				match (self, other) {
					$(
						($first, $second) => $result,
						($second, $first) => $result,
					)+
					_ => other,
				}
			};
		}

		one_of! {
			(North, East) => Northeast,
			(North, West) => Northwest,
			(South, East) => Southeast,
			(South, West) => Southwest,
		}
	}

	#[must_use]
	pub fn reduce(self, other: Self) -> Option<Self> {
		use Ordinal::*;

		if self == other {
			return None;
		}

		macro_rules! one_of {
			{ $($source:ident => ($first:ident, $second:ident)),+ $(,)? } => {
				match (self, other) {
					$(
						($source, $first) => Some($second),
						($source, $second) => Some($first),
					)+
					_ => Some(self),
				}
			};
		}

		one_of! {
			Northeast => (North, East),
			Northwest => (North, West),
			Southeast => (South, East),
			Southwest => (South, West),
		}
	}

	#[must_use]
	pub fn as_point<I: From<f32>>(&self) -> (I, I) {
		use std::f32::consts::SQRT_2;
		use Ordinal::*;

		const DIAG: f32 = 1.0 / SQRT_2;

		let (x, y) = match self {
			North => (0.0, -1.0),
			South => (0.0, 1.0),
			East => (-1.0, 0.0),
			West => (1.0, 0.0),
			Northeast => (-DIAG, -DIAG),
			Northwest => (DIAG, -DIAG),
			Southeast => (-DIAG, DIAG),
			Southwest => (DIAG, DIAG),
		};
		(x.into(), y.into())
	}
}

impl From<Cardinal> for Ordinal {
	fn from(dir: Cardinal) -> Self {
		use Cardinal::*;
		match dir {
			North => Ordinal::North,
			South => Ordinal::South,
			East => Ordinal::East,
			West => Ordinal::West,
		}
	}
}
