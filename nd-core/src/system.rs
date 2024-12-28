
use std::any::Any;

use crate::world::World;

pub struct Commands {
	queue: SegQueue<Box<dyn DynCommand>>,
}

impl Commands {
	pub fn new() -> Self {
		Self {
			queue: SegQueue::new()
		}
	}
	pub fn push(&self, command: impl Command) {
		self.queue.push(Box::new(command));
	}
	pub fn run(&self, world: &mut World) {
		while let Some(command) = self.queue.pop() {
			command.run(world)
		}
	}
}

pub trait Command: Any + Send + Sync {
	fn run(self, world: &mut World);
}

pub trait DynCommand: Any + Send + Sync {
	fn run(self: Box<Self>, world: &mut World);
}

impl<T: Command> DynCommand for T {
	fn run(self: Box<Self>, world: &mut World) {
		Command::run(*self, world)
	}
}

pub trait IntoSystemDescriptor {
	fn into(self) -> SystemDescriptor;
}

impl IntoSystemDescriptor for for<'a, 'b> fn(&'a World, &'b Commands) {
	fn into(self) -> SystemDescriptor {
		SystemDescriptor::new(self)
	}
}

pub struct SystemDescriptor {
	callback: for<'a, 'b> fn(&'a World, &'b Commands),
}

impl SystemDescriptor {
	pub fn new(callback: for<'a, 'b> fn(&'a World, &'b Commands)) -> Self {
		Self { callback }
	}
}

pub struct Systems {
	systems: Vec<SystemDescriptor>,
}
use crossbeam::queue::SegQueue;

impl Systems {
	pub fn new() -> Self {
		Self {
			systems: Vec::new(),
		}
	}
	pub fn add_system(&mut self, system: impl IntoSystemDescriptor) {
		self.systems.push(system.into())
	}
	pub async fn run(&self, world: &World, commands: &Commands) {
		for system in self.systems.iter() {


			(system.callback)(world, commands)

		}





	}
}
