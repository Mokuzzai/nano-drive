use serde::Deserialize;
use serde::Serialize;

#[derive(Debug, Serialize, Deserialize)]
#[non_exhaustive]
pub enum ClientEvent {
	CloseRequested,
	BeginAction { id: u8, value: Value },
	EndAction { id: u8 },
}

#[derive(Debug, Serialize, Deserialize)]
pub enum Value {
	Digital,
	Analog { x: f32 },
	Velocity { x: f32 },
	Velocity2 { x: f32, y: f32 },
	Position { x: f32 },
	Position2 { x: f32, y: f32 },
}
