use std::sync::LazyLock;

use crate::world::World;

use crossbeam::channel;
use crossbeam::channel::Receiver;
use crossbeam::channel::Sender;

use rayon::ThreadPool;

pub trait System: 'static + Send + Sync {
	fn run(&mut self, world: &World, commands: &mut Commands);
}

pub trait Command: 'static + Send + Sync {
	fn run(self: Box<Self>, world: &mut World);
}

type BoxedSystem = Box<dyn System>;
type BoxedCommand = Box<dyn Command>;

pub struct Commands<'a> {
	sender: &'a Sender<BoxedCommand>,
}

impl<'a> Commands<'a> {
	fn add(&mut self, command: impl Command) {
		self.sender.send(Box::new(command)).unwrap()
	}
}

struct Systems {
	systems: Vec<BoxedSystem>,
}

struct Channels {
	receiver: Receiver<BoxedCommand>,

	sender: Sender<BoxedCommand>,
}

impl Channels {
	fn new() -> Self {
		let (sender, receiver) = channel::unbounded();

		Self { receiver, sender }
	}
}

static CHANNEL: LazyLock<Channels> = LazyLock::new(Channels::new);

thread_local! {
	static SENDER: Sender<BoxedCommand> = CHANNEL.sender.clone();
}

impl Systems {
	fn run(&mut self, thread_pool: &ThreadPool, world: &World) {
		assert!(CHANNEL.receiver.is_empty());

		thread_pool.scope(|scope| {
			for system in self.systems.iter_mut() {
				scope.spawn(|_| {
					SENDER.with(|sender| {
						assert!(sender.is_empty());

						system.run(world, &mut Commands { sender });
					})
				})
			}
		})
	}
}
