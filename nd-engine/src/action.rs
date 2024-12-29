use fxhash::FxHashMap;
use serde::Deserialize;
use serde::Serialize;

#[derive(Debug, Serialize, Deserialize)]
pub enum Value {
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
	actions: FxHashMap<String, Value>,
}
