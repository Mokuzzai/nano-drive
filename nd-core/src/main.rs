#![allow(unused)]

use std::sync::mpsc::channel;
use std::thread;
use std::time::Duration;
use std::time::Instant;

use glium::backend::glutin::SimpleWindowBuilder;
use glium::glutin::surface::WindowSurface;
use glium::winit;
use glium::winit::event::DeviceEvent;
use glium::Surface;
use nalgebra::QR;
use winit::application::ApplicationHandler;
use winit::event::DeviceId;
use winit::event::WindowEvent;
use winit::event_loop;
use winit::event_loop::ActiveEventLoop;
use winit::window::Window;
use winit::window::WindowId;

use crossbeam::channel;
use crossbeam::channel::Receiver;
use crossbeam::channel::Sender;

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

#[derive(Debug)]
enum ApplicationEvent {
	DeviceEvent(DeviceEvent),
	CloseRequested,
}

#[derive(Debug)]
enum EngineEvent {
	Closed,
}

struct Engine {}

impl Engine {
	fn new() -> Self {
		Self {}
	}
	fn handle_app_event(&mut self, event: ApplicationEvent, msg: &mut Sender<EngineEvent>) -> bool {
		println!("ApplicationEvent::{:?}", event);

		match event {
			ApplicationEvent::CloseRequested => return true,
			ApplicationEvent::DeviceEvent(event) => match event {
				_ => (),
			},
			_ => (),
		}

		false
	}
	fn handle_fixed_update(&mut self, msg: &mut Sender<EngineEvent>) {}
}

fn run_engine(
	mut app_receiver: Receiver<ApplicationEvent>,
	mut engine_sender: Sender<EngineEvent>,
) {
	let mut engine = Engine::new();
	let mut running = true;
	let mut last_fixed_update = Instant::now();

	let wait_time = Duration::from_secs(1) / 60;

	println!("Engine launched");

	while running {
		let deadline = last_fixed_update + wait_time;

		while let Ok(event) = app_receiver.recv_deadline(deadline) {
			if engine.handle_app_event(event, &mut engine_sender) {
				running = false
			}
		}

		last_fixed_update = Instant::now();

		engine.handle_fixed_update(&mut engine_sender);
	}

	println!("Engine shutdown");
}

struct EngineHandle {
	app_sender: Sender<ApplicationEvent>,
	engine_receiver: Receiver<EngineEvent>,
}

impl EngineHandle {
	fn launch() -> Self {
		let (app_sender, app_receiver) = channel::unbounded();
		let (engine_sender, engine_receiver) = channel::unbounded();

		thread::spawn(|| {
			run_engine(app_receiver, engine_sender);
		});

		Self {
			app_sender,
			engine_receiver,
		}
	}
	fn send(&mut self, event: ApplicationEvent) {
		let Err(error) = self.app_sender.send(event) else {
			return;
		};

		let event = error.into_inner();

		*self = Self::launch();

		self.app_sender.send(event).unwrap();
	}
}

struct Application {
	render_server: RenderServer,

	engine: Option<EngineHandle>,

	last_draw: Instant,
}

impl Application {
	fn new(event_loop: &EventLoop) -> Self {
		Self {
			render_server: {
				let (window, display) = SimpleWindowBuilder::new().build(event_loop);

				RenderServer { window, display }
			},
			engine: Some(EngineHandle::launch()),
			last_draw: Instant::now(),
		}
	}
	fn handle_engine_event(&mut self, event: EngineEvent, event_loop: &ActiveEventLoop) {
		println!("EngineEvent::{:?}", event);

		match event {
			EngineEvent::Closed => event_loop.exit(),
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
				if let Some(mut engine) = self.engine.take() {
					engine.send(ApplicationEvent::CloseRequested)
				} else {
					event_loop.exit();
				}
			}
			_ => (),
		}
	}

	fn about_to_wait(&mut self, event_loop: &ActiveEventLoop) {
		let wait_time = Duration::from_secs(1) / 60;

		if let Some(engine) = self.engine.as_mut() {
			if let Ok(event) = engine.engine_receiver.try_recv() {
				self.handle_engine_event(event, event_loop);
			}
		} else {
			event_loop.exit();
		}

		// println!("{:?}", self.last_draw.elapsed());

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
		if let Some(engine) = self.engine.as_mut() {
			engine.send(ApplicationEvent::DeviceEvent(event))
		}
	}
}

fn main() {
	let event_loop = winit::event_loop::EventLoop::builder().build().unwrap();

	let mut app = Application::new(&event_loop);

	event_loop.run_app(&mut app).unwrap();
}
