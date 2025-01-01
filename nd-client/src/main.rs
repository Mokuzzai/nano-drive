#![allow(unused)]

mod input;

use nd_engine::client::ClientEvent;
use nd_engine::engine;
use nd_engine::engine::EngineEvent;
use nd_engine::engine::EngineHandle;

use std::env;
use std::fs;
use std::path::PathBuf;
use std::sync::mpsc::channel;
use std::time::Duration;
use std::time::Instant;

use clap::Parser;
use glium::Surface;
use glium::backend::glutin::SimpleWindowBuilder;
use glium::glutin::surface::WindowSurface;
use glium::winit;
use glium::winit::event::DeviceEvent;
use serde_json as json;
use winit::application::ApplicationHandler;
use winit::event::DeviceId;
use winit::event::WindowEvent;
use winit::event_loop;
use winit::event_loop::ActiveEventLoop;
use winit::window::Window;
use winit::window::WindowId;

type EventLoop = event_loop::EventLoop<UserEvent>;
type Display = glium::Display<WindowSurface>;
type UserEvent = ();

struct RenderServer {
	window: Window,
	display: Display,
}

impl RenderServer {
	fn redraw(&mut self) {
		let mut frame = self.display.draw();

		frame.clear_all((100.0 / 255.0, 149.0 / 255.0, 237.0 / 255.0, 1.0), 0.0, 0);

		frame.finish().unwrap();
	}
}

struct Client {
	render_server: RenderServer,

	engine: Option<EngineHandle>,

	last_draw: Instant,
}

impl Client {
	fn new(event_loop: &EventLoop, settings: ClientConfig) -> Self {
		Self {
			render_server: {
				let (window, display) = SimpleWindowBuilder::new().build(event_loop);

				RenderServer { window, display }
			},
			engine: Some(EngineHandle::spawn(settings.engine_path)),
			last_draw: Instant::now(),
		}
	}
	fn handle_engine_event(&mut self, event: EngineEvent, event_loop: &ActiveEventLoop) {
		match event {
			EngineEvent::Closed => {
				let _ = self.engine.take();
			}
			_ => (),
		}
	}
}

impl ApplicationHandler for Client {
	fn resumed(&mut self, event_loop: &ActiveEventLoop) {}

	fn window_event(
		&mut self,
		event_loop: &ActiveEventLoop,
		window_id: WindowId,
		event: WindowEvent,
	) {
		assert!(window_id == self.render_server.window.id());

		match event {
			WindowEvent::RedrawRequested => self.render_server.redraw(),
			WindowEvent::CloseRequested => {
				if let Some(engine) = self.engine.as_mut() {
					engine.send(ClientEvent::CloseRequested);
				}
			}
			_ => (),
		}
	}

	fn about_to_wait(&mut self, event_loop: &ActiveEventLoop) {
		if let Some(engine) = self.engine.as_mut() {
			if let Some(event) = engine.try_recv() {
				self.handle_engine_event(event, event_loop);
			}
		} else {
			event_loop.exit();
		}

		let wait_time = Duration::from_secs(1) / 60;

		if self.last_draw.elapsed() > wait_time {
			self.render_server.window.request_redraw();

			self.last_draw = Instant::now();
		}

		event_loop.set_control_flow(event_loop::ControlFlow::Poll);
	}

	fn device_event(
		&mut self,
		event_loop: &ActiveEventLoop,
		device_id: DeviceId,
		event: DeviceEvent,
	) {
	}
}

#[derive(Parser)]
pub struct LaunchOptions {
	engine_path: PathBuf,
}

#[derive(Debug)]
pub struct ClientConfig {
	engine_path: PathBuf,
}

fn main() {
	let launch_options = LaunchOptions::parse();

	let application_config = ClientConfig {
		engine_path: launch_options.engine_path,
	};

	println!("{:#?}", application_config);

	let event_loop = EventLoop::builder().build().unwrap();

	let mut app = Client::new(&event_loop, application_config);

	event_loop.run_app(&mut app).unwrap();
}
