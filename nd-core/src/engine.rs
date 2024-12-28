use crate::plugin::Plugins;
use crate::system::Commands;
use crate::system::Systems;
use crate::world::World;
use crate::ApplicationEvent;

use std::fs;
use std::path::Path;
use std::path::PathBuf;
use std::process::Child;
use std::process::Command;
use std::thread;
use std::time::Duration;
use std::time::Instant;

use ipc_channel::ipc;
use ipc_channel::ipc::IpcOneShotServer;
use ipc_channel::ipc::IpcReceiver;
use ipc_channel::ipc::IpcSender;
use serde::Deserialize;
use serde::Serialize;
use serde_json as json;

#[derive(Clone, Serialize, Deserialize)]
pub struct PluginConfig {
	path: PathBuf,
	enabled: bool,
}

#[derive(Clone, Serialize, Deserialize)]
pub struct EngineConfig {
	plugins: Vec<PluginConfig>,
}

#[derive(Clone, Serialize, Deserialize)]
pub struct EngineBuilder {
	pipe: String,
	engine_config: EngineConfig,
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
pub enum EngineEvent {
	Closed,
}

pub struct Engine {
	engine_config: EngineConfig,
	app_receiver: IpcReceiver<ApplicationEvent>,
	engine_sender: IpcSender<EngineEvent>,
	world: World,
	plugins: Plugins,
	startup_systems: Systems,
	systems: Systems,
}

impl Engine {
	pub fn new(
		engine_config: EngineConfig,
		engine_sender: IpcSender<EngineEvent>,
		app_receiver: IpcReceiver<ApplicationEvent>,
	) -> Self {
		Self {
			engine_config,
			app_receiver,
			engine_sender,
			world: World::new(),
			startup_systems: Systems::new(),
			systems: Systems::new(),
			plugins: Plugins::new(),
		}
	}

	pub fn run(mut self) {
		let mut running = true;
		let mut last_fixed_update = Instant::now();

		let wait_time = Duration::from_secs(1) / 60;

		let runtime = tokio::runtime::Runtime::new().unwrap();

		println!("Engine launched");

		let commands = Commands::new();

		runtime.block_on(self.startup_systems.run(&self.world, &commands));

		commands.run(&mut self.world);

		while running {
			loop {
				let timeout = wait_time.saturating_sub(last_fixed_update.elapsed());

				match self.app_receiver.try_recv_timeout(timeout) {
					Ok(event) => self.world.app_events.push(event),
					Err(_) => break,
				}
			}

			last_fixed_update = Instant::now();

			runtime.block_on(self.systems.run(&self.world, &commands));

			commands.run(&mut self.world);
		}

		self.engine_sender.send(EngineEvent::Closed);

		println!("Engine shutdown");
	}
}

#[derive(Serialize, Deserialize)]
struct Connect {
	app_sender: IpcSender<ApplicationEvent>,
	engine_receiver: IpcReceiver<EngineEvent>,
}

pub struct EngineHandle {
	pub app_sender: IpcSender<ApplicationEvent>,
	pub engine_receiver: IpcReceiver<EngineEvent>,
	pub process: Child,
}

impl EngineHandle {
	pub fn spawn(engine_path: &Path, engine_config: EngineConfig) -> Self {
		let (sender, name) = IpcOneShotServer::new().unwrap();

		let builder = EngineBuilder {
			pipe: name,
			engine_config,
		};

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
