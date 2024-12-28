#![allow(unused)]

use nd_core::engine;
use nd_core::engine::EngineConfig;
use nd_core::engine::EngineEvent;
use nd_core::engine::EngineHandle;
use nd_core::ApplicationEvent;

use std::env;
use std::fs;
use std::path::PathBuf;
use std::sync::mpsc::channel;
use std::time::Duration;
use std::time::Instant;

use clap::Parser;
use glium::backend::glutin::SimpleWindowBuilder;
use glium::glutin::surface::WindowSurface;
use glium::winit;
use glium::winit::event::DeviceEvent;
use glium::Surface;
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

struct Application {
	render_server: RenderServer,

	application_config: ApplicationConfig,

	engine: Option<EngineHandle>,

	last_draw: Instant,
}

impl Application {
	fn new(event_loop: &EventLoop, settings: ApplicationConfig) -> Self {
		Self {
			render_server: {
				let (window, display) = SimpleWindowBuilder::new().build(event_loop);

				RenderServer { window, display }
			},
			engine: Some(EngineHandle::spawn(
				&settings.engine_path,
				settings.engine_config.clone(),
			)),
			application_config: settings,
			last_draw: Instant::now(),
		}
	}
	fn handle_engine_event(&mut self, event: EngineEvent, event_loop: &ActiveEventLoop) {
		println!("EngineEvent::{:?}", event);

		match event {
			EngineEvent::Closed => {
				let _ = self.engine.take();
			}
			_ => (),
		}
	}
}

impl ApplicationHandler for Application {
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
					let _ = engine
						.app_sender
						.send(ApplicationEvent::CloseRequested)
						.unwrap();
				}
			}
			_ => (),
		}
	}

	fn about_to_wait(&mut self, event_loop: &ActiveEventLoop) {
		if let Some(engine) = self.engine.as_mut() {
			match engine.process.try_wait() {
				Ok(None) => (), // do nothing
				Ok(Some(exit_code)) => {
					println!("engine exited with code: {:?}", exit_code);

					*engine = EngineHandle::spawn(
						&self.application_config.engine_path,
						self.application_config.engine_config.clone(),
					);
				}
				Err(e) => panic!("{}", e),
			}

			if let Ok(event) = engine.engine_receiver.try_recv() {
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

pub struct ApplicationConfig {
	engine_path: PathBuf,
	engine_config: EngineConfig,
}

fn main() {
	let mut path = env::current_exe().unwrap();

	let _ = path.pop();

	let engine_path = path.join("nd-sim.exe");

	let engine_config_path = path.join("project.json");

	let settings = ApplicationConfig {
		engine_path,
		engine_config: json::from_reader(fs::File::open(&engine_config_path).unwrap_or_else(
			|error| {
				panic!(
					"could not open file: {}, reason: {}",
					engine_config_path.display(),
					error
				)
			},
		))
		.unwrap(),
	};

	let event_loop = EventLoop::builder().build().unwrap();

	let mut app = Application::new(&event_loop, settings);

	event_loop.run_app(&mut app).unwrap();
}
