use crate::client::ClientEvent;
use crate::system::System;
use crate::system::Systems;
use crate::world::World;

use std::path::Path;
use std::path::PathBuf;
use std::process::Child;
use std::process::Command;
use std::time::Duration;
use std::time::Instant;

use ipc_channel::ipc;
use ipc_channel::ipc::IpcOneShotServer;
use ipc_channel::ipc::IpcReceiver;
use ipc_channel::ipc::IpcSender;
use ipc_channel::ipc::TryRecvError;
use rayon::ThreadPool;
use rayon::ThreadPoolBuilder;
use serde::Deserialize;
use serde::Serialize;
use serde_json as json;

#[derive(Debug, Serialize, Deserialize)]
pub struct EngineBuilder {
	pipe: String,
}

#[derive(Debug, Serialize)]
pub struct EngineBuilderRef<'a> {
	pipe: &'a str,
}

impl EngineBuilder {
	pub fn build(self) -> Engine {
		let sender = IpcSender::connect(self.pipe).unwrap();

		let (engine_sender, engine_receiver) = ipc::channel().unwrap();
		let (app_sender, app_receiver) = ipc::channel().unwrap();

		sender
			.send(Connect {
				app_sender,
				engine_receiver,
			})
			.unwrap();

		Engine::new(engine_sender, app_receiver)
	}
}

#[derive(Debug, Serialize, Deserialize)]
#[non_exhaustive]
pub enum EngineEvent {
	Closed,
	BeginFixedUpdate(u64),
	EndFixedUpdate(u64),
}

pub struct Engine {
	app_receiver: IpcReceiver<ClientEvent>,
	engine_sender: IpcSender<EngineEvent>,
	world: World,
	thread_pool: ThreadPool,
	startup_systems: Systems,
	systems: Systems,
	ticks_per_second: u32,
}

impl Engine {
	pub fn new(
		engine_sender: IpcSender<EngineEvent>,
		app_receiver: IpcReceiver<ClientEvent>,
	) -> Self {
		Self {
			app_receiver,
			engine_sender,
			world: World::new(),
			thread_pool: ThreadPoolBuilder::new().num_threads(16).build().unwrap(),
			startup_systems: Systems::new(),
			systems: Systems::new(),
			ticks_per_second: 120,
		}
	}
	pub fn add_startup_system(&mut self, system: impl System) -> &mut Self {
		self.startup_systems.add(system);
		self
	}
	pub fn add_system(&mut self, system: impl System) -> &mut Self {
		self.systems.add(system);
		self
	}
	pub fn run(&mut self) {
		let mut running = true;
		let mut last_fixed_update = Instant::now();
		let mut tick = 0;

		let wait_time = Duration::from_secs(1) / self.ticks_per_second;

		println!("Engine start");

		self.world
			.write_swap_sync(&self.thread_pool, &mut self.startup_systems);

		println!("ran startup systems");

		while running {
			loop {
				let timeout = wait_time.saturating_sub(last_fixed_update.elapsed());

				match self.app_receiver.try_recv_timeout(timeout) {
					Ok(ClientEvent::CloseRequested) => running = false,
					Ok(_event) => {
						// self.world.app_events.push(event)
					}
					Err(TryRecvError::Empty) => break,
					Err(error) => panic!("error while reciving application event: {}", error),
				}
			}

			last_fixed_update = Instant::now();

			self.engine_sender
				.send(EngineEvent::BeginFixedUpdate(tick))
				.unwrap();

			self.world
				.write_swap_sync(&self.thread_pool, &mut self.systems);

			self.engine_sender
				.send(EngineEvent::EndFixedUpdate(tick))
				.unwrap();

			tick += 1;
		}

		self.engine_sender.send(EngineEvent::Closed).unwrap();

		println!("Engine exit");
	}
}

#[derive(Serialize, Deserialize)]
struct Connect {
	app_sender: IpcSender<ClientEvent>,
	engine_receiver: IpcReceiver<EngineEvent>,
}

struct RawEngineHandle {
	app_sender: IpcSender<ClientEvent>,
	engine_receiver: IpcReceiver<EngineEvent>,
	process: Child,
}

impl RawEngineHandle {
	fn spawn(engine_path: &Path) -> Self {
		let (sender, name) = IpcOneShotServer::new().unwrap();

		let builder = EngineBuilderRef { pipe: &name };

		println!("spawning engine with: {:#?}", builder);

		let builder = json::to_string(&builder).unwrap();

		let process = Command::new(engine_path)
			.arg(&builder)
			.spawn()
			.unwrap_or_else(|error| {
				panic!(
					"error while running command: {}, reason: {}",
					engine_path.display(),
					error
				);
			});

		let Connect {
			app_sender,
			engine_receiver,
		} = sender.accept().unwrap().1;

		Self {
			app_sender,
			engine_receiver,
			process,
		}
	}
}

pub struct EngineHandle {
	engine_path: PathBuf,
	raw: RawEngineHandle,
}

impl EngineHandle {
	pub fn spawn(engine_path: PathBuf) -> Self {
		let raw = RawEngineHandle::spawn(&engine_path);

		Self { engine_path, raw }
	}

	fn rebuild_if_needed(&mut self) {
		let did_exit = self.raw.process.try_wait().unwrap_or_else(|error| {
			panic!("error while waiting for child: {}", error);
		});

		if let Some(exit_code) = did_exit {
			println!("child exited with code: {:?}", exit_code.code());

			self.raw = RawEngineHandle::spawn(&self.engine_path);
		}
	}

	pub fn send(&mut self, client_event: ClientEvent) {
		self.rebuild_if_needed();

		self.raw
			.app_sender
			.send(client_event)
			.unwrap_or_else(|error| {
				panic!("error while sending application event: {}", error);
			});
	}

	pub fn try_recv(&mut self) -> Option<EngineEvent> {
		match self.raw.engine_receiver.try_recv() {
			Ok(event) => Some(event),
			Err(TryRecvError::Empty) => None,
			Err(error) => panic!("error while receiving engine event: {}", error),
		}
	}
}
