use crate::action::Actions;
use crate::client::ClientEvent;
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
use serde::Deserialize;
use serde::Serialize;
use serde_json as json;

fn bool_true() -> bool {
	true
}

#[derive(Debug, Serialize, Deserialize)]
pub struct PluginConfig {
	path: PathBuf,
	#[serde(default = "bool_true")]
	enabled: bool,
}

#[derive(Debug, Serialize, Deserialize)]
pub struct EngineConfig {
	plugins: Vec<PluginConfig>,
	actions: Actions,
}

#[derive(Debug, Serialize, Deserialize)]
pub struct EngineBuilder {
	pipe: String,
	engine_config: EngineConfig,
}

#[derive(Debug, Serialize)]
pub struct EngineBuilderRef<'a> {
	pipe: &'a str,
	engine_config: &'a EngineConfig,
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

		Engine::new(self.engine_config, engine_sender, app_receiver)
	}
}

#[derive(Debug, Serialize, Deserialize)]
#[non_exhaustive]
pub enum EngineEvent {
	Closed,
}

pub struct Engine {
	engine_config: EngineConfig,
	app_receiver: IpcReceiver<ClientEvent>,
	engine_sender: IpcSender<EngineEvent>,
	world: World,
}

impl Engine {
	pub fn new(
		engine_config: EngineConfig,
		engine_sender: IpcSender<EngineEvent>,
		app_receiver: IpcReceiver<ClientEvent>,
	) -> Self {
		Self {
			engine_config,
			app_receiver,
			engine_sender,
			world: World::new(),
		}
	}

	pub fn run(mut self) {
		let mut running = true;
		let mut last_fixed_update = Instant::now();

		let wait_time = Duration::from_secs(1) / 60;

		println!("Engine start");

		while running {
			loop {
				let timeout = wait_time.saturating_sub(last_fixed_update.elapsed());

				match self.app_receiver.try_recv_timeout(timeout) {
					Ok(event) => {
						println!("engine received: {:?}", event);

						if matches!(event, ClientEvent::CloseRequested) {
							running = false
						}

						self.world.app_events.push(event)
					}
					Err(TryRecvError::Empty) => break,
					Err(error) => panic!("error while reciving application event: {}", error),
				}
			}

			last_fixed_update = Instant::now();
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
	fn spawn(engine_path: &Path, engine_config: &EngineConfig) -> Self {
		let (sender, name) = IpcOneShotServer::new().unwrap();

		let builder = EngineBuilderRef {
			pipe: &name,
			engine_config,
		};

		println!("spawning engine with: {:#?}", builder);

		let builder = json::to_string(&builder).unwrap();

		let process = Command::new(engine_path).arg(&builder).spawn().unwrap();

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
	engine_config: EngineConfig,
	raw: RawEngineHandle,
}

impl EngineHandle {
	pub fn spawn(engine_path: PathBuf, engine_config: EngineConfig) -> Self {
		let raw = RawEngineHandle::spawn(&engine_path, &engine_config);

		Self {
			engine_path,
			engine_config,
			raw,
		}
	}

	fn rebuild_if_needed(&mut self) {
		let did_exit = self.raw.process.try_wait().unwrap_or_else(|error| {
			panic!("error while waiting for child: {}", error);
		});

		if let Some(exit_code) = did_exit {
			println!("child exited with code: {:?}", exit_code.code());

			self.raw = RawEngineHandle::spawn(&self.engine_path, &self.engine_config);
		}
	}

	pub fn send(&mut self, client_event: ClientEvent) {
		println!("client sent: {:?}", client_event);

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
