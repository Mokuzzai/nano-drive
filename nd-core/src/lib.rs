#![allow(unused)]

pub mod engine;
pub mod plugin;
pub mod system;
pub mod world;

pub use engine::EngineEvent;

use serde::Deserialize;
use serde::Serialize;

use glium::winit;

use winit::event::DeviceEvent;

#[derive(Debug, Serialize, Deserialize)]
pub enum ApplicationEvent {
	CloseRequested,
}
