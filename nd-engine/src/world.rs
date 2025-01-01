use std::mem;

use crate::system::Systems;

use crossbeam::queue::SegQueue;
use rayon::ThreadPool;

#[derive(Clone)]
pub struct RawWorld {}

impl RawWorld {
	pub fn new() -> Self {
		RawWorld {}
	}
}

pub struct EventWriter<T> {
	write: Vec<T>,
}

impl<T> EventWriter<T> {
	pub fn new() -> Self {
		Self { write: Vec::new() }
	}
}

pub struct EventBuffer<T> {
	read: Vec<T>,
}

impl<T> EventBuffer<T> {
	pub fn new() -> Self {
		Self { read: Vec::new() }
	}
	pub fn iter(&self) -> impl Iterator<Item = &T> {
		self.read.iter()
	}
}

struct WorldAndCommands {
	world: RawWorld,
	commands: SegQueue<BoxedCommand>,
}

impl WorldAndCommands {
	fn new() -> Self {
		Self {
			world: RawWorld::new(),
			commands: SegQueue::new(),
		}
	}
}

pub struct World {
	src: WorldAndCommands,
	dst: WorldAndCommands,
}

impl World {
	pub fn new() -> Self {
		Self {
			src: WorldAndCommands::new(),
			dst: WorldAndCommands::new(),
		}
	}
	pub fn write_swap_sync<'a>(&mut self, thread_pool: &ThreadPool, systems: &mut Systems) {
		assert!(self.dst.commands.is_empty());

		thread_pool.scope(|scope| {
			scope.spawn(|_| {
				while let Some(command) = self.dst.commands.pop() {
					command.run(&mut self.dst.world);
				}
			});

			for system in systems.iter_mut() {
				scope.spawn(|_| {
					system.run(&self.src.world, &mut Commands {
						queue: &self.src.commands,
					})
				});
			}
		});

		self.src.world.clone_from(&self.dst.world);

		mem::swap(&mut self.src.world, &mut self.dst.world)
	}
}

pub trait Command: 'static + Send + Sync {
	fn run(self: Box<Self>, world: &mut RawWorld);
}

pub struct Commands<'a> {
	queue: &'a SegQueue<BoxedCommand>,
}
type BoxedCommand = Box<dyn Command>;

impl Commands<'_> {
	pub fn add(&mut self, command: impl Command) {
		self.queue.push(Box::new(command))
	}
}
