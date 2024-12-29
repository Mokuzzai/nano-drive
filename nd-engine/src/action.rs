use fxhash::FxHashMap;
use serde::Deserialize;
use serde::Serialize;

#[derive(Debug, Serialize, Deserialize)]
pub enum Kind {
	// Button
	Absolute,

	// Switch between -1..1
	AbsoluteAxis,

	// D-Pad, 4 Buttons
	AbsoluteAxis2,

	PositionAxis,

	// Mouse cursor
	PositionAxis2,

	VelocityAxis,

	// Mouse motion
	// Mouse wheel
	VelocityAxis2,
}

#[derive(Debug, Serialize, Deserialize)]
pub struct Actions {
	#[serde(flatten)]
	actions: FxHashMap<String, Kind>,
}

impl Actions {
	pub fn new() -> Self {
		Self {
			actions: FxHashMap::default(),
		}
	}
	pub fn insert(&mut self, name: String, kind: Kind) -> Option<Kind> {
		self.actions.insert(name, kind)
	}
}
